const response = await fetch("api/auth", {
  method: "POST",
  headers: {
    "Content-Type": "application/json"
  },
  body: JSON.stringify({nick, pswd})
});

if (response.ok) {
  window.location.href = "/";
} else {
  const e = await response.text();
  console.log(e);
}
