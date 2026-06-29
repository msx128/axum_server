const button = document.getElementById('clickBtn');
const counter = document.getElementById('counter');

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
  fetch("http://127.0.0.1:8080/api/click", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(meta_data)
  })
  .then(response => response.text())
  .then(data => {
    counter.textContent = data;
  });
}

button.onclick = sendOnClick;
