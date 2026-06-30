# Click Counter

A simple full-stack web application built with Rust, Axum, SQLx, and PostgreSQL.

This project was created to explore modern backend development concepts such as authentication, session management, database integration, and frontend-backend communication. The application allows users to register, log in, and track their personal click count.

## Features

* User registration and login
* Cookie-based session authentication
* PostgreSQL database storage
* Persistent click counter per user
* Static frontend served by the backend
* Asynchronous request handling with Axum

## Tech Stack

### Backend

* Rust
* Axum
* SQLx
* PostgreSQL
* Tokio

### Frontend

* HTML
* CSS
* JavaScript

## Running the Project

1. Create a PostgreSQL database.
2. Configure the database connection string.
3. Run database migrations.
4. Start the server:

```bash
cargo run
```

The application will be available at:

```text
http://localhost:3000
```

## Purpose

This project is primarily a learning project focused on backend development. The goal was to gain practical experience with web servers, authentication, database interactions, and application state management while working with Rust and its ecosystem.

## Future Improvements

* Password hashing with Argon2
* Session expiration and logout functionality
* Better error handling
* Automated tests
* Docker support
* Improved project structure and service separation

---

Built with Rust.
