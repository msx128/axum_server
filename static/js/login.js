const form = document.querySelector(".login-form");

form.addEventListener("submit", async (e) => {
  e.preventDefault();

  const nick = document.getElementById("username").value;
  const pswd = document.getElementById("password").value;

  try {
    const response = await fetch ("/api/singin", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({nick, pswd})
    });

    console.log("Response status:", response.status);
    console.log("Response OK:", response.ok);
    
    const data = await response.json();
    console.log("Server response:", data);

    if (response.ok) {
      window.location.href = "/";
    } else {
      console.log("Error:", response.status, response.statusText);
    }
  } catch (error) {
    console.error("fetch error:", error);
  }
});
