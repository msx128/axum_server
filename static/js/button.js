const button = document.getElementById('clickBtn');
const counter = document.getElementById('counter');
const global = document.getElementById('global');

class MetaDataCollector {
  constructor(message) {
    this.message = message;
    this.date = new Date().toISOString();
    this.userAgent = navigator.userAgent;
  }

  getData() {
    return {
      message: this.message,
      date: this.date,
      userAgent: this.userAgent,
    };
  }
}

const collector = new MetaDataCollector('hi');
const meta_data = collector.getData();

function sendOnClick() {
  fetch("http://localhost:3000/api/click", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(meta_data)
  })
  .then(response => response.json())
  .then(data => {
    counter.textContent = data.user_clicks;
    global.textContent = data.global_clicks;
  });
}

button.onclick = sendOnClick;
