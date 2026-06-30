use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Extension, Json,
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use dotenvy::dotenv;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
};
use std::env;
use std::{str::FromStr, sync::Arc};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::services::{ServeDir, ServeFile};

struct Database {
    pool: PgPool,
}

const LIFESPAN: i32 = 52;

impl Database {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_opt = PgConnectOptions::from_str(&db_url)?.to_owned();

        let pool = PgPoolOptions::new().connect_with(db_opt).await?;

        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            nick TEXT NOT NULL UNIQUE,
            pswd TEXT NOT NULL,
            clicks INT NOT NULL DEFAULT 0);",
        )
        .execute(&pool)
        .await?;

        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS sessions (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id),
                token TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

fn create_password_hash(pswd: String) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let hash = argon2.hash_password(&pswd.into_bytes(), &salt)?;
    Ok(hash.to_string())
}

#[derive(Serialize, Deserialize)]
struct CreateUser {
    nick: String,
    pswd: String,
}

#[derive(Serialize, Deserialize)]
struct UserResponse {
    id: i32,
    token: String,
}

#[derive(FromRow, Serialize, Deserialize)]
struct CreatedUser {
    id: i32,
}

async fn create_user(db: &Database, user: CreateUser) -> Result<i32, sqlx::Error> {
    let pswd =
        create_password_hash(user.pswd).map_err(|_| return sqlx::error::Error::WorkerCrashed)?;
    // 100% need custom errors or other way deal with them
    // withoud returning bs
    let created = sqlx::query_as!(
        CreatedUser,
        r#"
            INSERT INTO users (nick, pswd)
            VALUES ($1, $2)
            RETURNING id; 
        "#,
        user.nick,
        pswd // we need to store password in hash and when we want to check it use PasswordHash::new()
    )
    .fetch_one(&db.pool)
    .await?;

    Ok(created.id)
}

async fn insert_id_token(db: &Database, user: &UserResponse) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
            INSERT INTO sessions (user_id, token)
            VALUES ($1, $2);
        "#,
        user.id,
        user.token
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}

// we need logout, mb make just cookie eraser, that should work

enum LoginError {
    UserNotFound,
    InvalidPassword,
    DatabaseError,
    PasswordHashError,
}

#[derive(FromRow)]
struct LoginUserCheck {
    id: i32,
    pswd: String,
}

async fn check_login(db: &Database, user: &CreateUser) -> Result<i32, LoginError> {
    let found = sqlx::query_as!(
        LoginUserCheck,
        "SELECT id, pswd FROM users WHERE nick = $1",
        user.nick
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => LoginError::UserNotFound,
        _ => LoginError::DatabaseError,
    })?;

    let parsed_pswd_hash =
        PasswordHash::new(&found.pswd).map_err(|_| LoginError::PasswordHashError)?;

    if Argon2::default()
        .verify_password(&user.pswd.as_bytes(), &parsed_pswd_hash) // b"password" == hash
        .is_ok()
    {
        Ok(found.id)
    } else {
        Err(LoginError::InvalidPassword)
    }
}

async fn singin_handler(
    cookies: Cookies,
    Extension(db): Extension<Arc<Database>>,
    Json(user): Json<CreateUser>,
) -> Result<Json<UserResponse>, StatusCode> {
    println!("{} {}", &user.nick, &user.pswd);

    let mut id: i32 = 0;
    let mut found: bool = true;

    match check_login(&db, &user).await {
        Ok(i) => id = i,
        Err(LoginError::UserNotFound) => found = false,
        Err(LoginError::InvalidPassword) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    if !found {
        id = create_user(&db, user).await.map_err(|_| {
            return StatusCode::INTERNAL_SERVER_ERROR;
        })?;
    }

    let token = make_rand_token();

    let push = UserResponse { id, token };

    insert_id_token(&db, &push).await.map_err(|_| {
        return StatusCode::INTERNAL_SERVER_ERROR;
    })?;

    let new_cookie: Cookie = Cookie::build(("session_token", push.token.clone()))
        .path("/")
        .http_only(true)
        .build();

    cookies.add(new_cookie);

    Ok(Json(push))
}

fn make_rand_token() -> String {
    // mb use OsRng so code more consistent
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);

    let token = URL_SAFE_NO_PAD.encode(buf);

    token
}

async fn button_page_handler(
    cookies: Cookies,
    Extension(db): Extension<Arc<Database>>,
) -> Result<axum::response::Html<String>, Redirect> {
    let cookie = cookies.get("session_token").ok_or(Redirect::to("/login"))?;

    let token = cookie.value();

    let lifespan: i32 = LIFESPAN;
    let _user_id = sqlx::query!(
        "
            SELECT user_id FROM sessions WHERE token = $1 AND created_at > (NOW() - ($2::integer || 'hours')::interval)
        ",                                                          // :: is a typecast, || is a string concatination 
        &token,
        lifespan,
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|_| Redirect::to("/login"))?; //don't need session, at least now

    println!("logged via {}", token);

    tokio::fs::read_to_string("./static/button/index.html")
        .await
        .map(axum::response::Html)
        .map_err(|_| Redirect::to("/login")) //may god forbid me for this sin
}

#[derive(Serialize)]
struct ClicksCounter {
    user_clicks: i32,
    global_clicks: i64,
}

async fn click_handler(
    Extension(db): Extension<Arc<Database>>,
    cookies: Cookies,
) -> Result<Json<ClicksCounter>, Redirect> {
    let cookie = cookies.get("session_token").ok_or(Redirect::to("/login"))?;

    let token = cookie.value();

    let session = sqlx::query!(
        "
            SELECT user_id FROM sessions WHERE token = $1 AND created_at < NOW()
        ",
        &token
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|_| Redirect::to("/login"))?;

    let u = sqlx::query_as!(
        ClicksCounter,
        r#"
        UPDATE users
        SET clicks = clicks + 1
        WHERE id = $1
        RETURNING
            clicks AS user_clicks,
            (SELECT COALESCE(SUM(clicks), 0) FROM users) AS "global_clicks!"
        "#,
        session.user_id
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|_| Redirect::to("/login"))?;

    Ok(Json(u))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Database::new().await?);

    let app = axum::Router::new()
        .route("/api/singin", post(singin_handler))
        .route("/api/click", post(click_handler))
        .route("/button", get(button_page_handler))
        .fallback_service(
            ServeDir::new("static").not_found_service(ServeFile::new("static/404.html")),
        )
        .layer(Extension(state))
        .layer(CookieManagerLayer::new());

    let listener = tokio::net::TcpListener::bind("localhost:3000").await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
