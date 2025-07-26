const API = "http://localhost:8080";

async function testClient() {
  const userId = "test-" + Math.random().toString(36).slice(2, 8);
  console.log(`Testing with user ID: ${userId}\n`);
  
  const messages = [
    "Hello!",
    "What's the weather in NYC?",
    "Remember my favorite color is blue",
    "What do you remember?",
    "15 * 24 + 8?"
  ];
  
  for (const message of messages) {
    try {
      const response = await fetch(`${API}/chat`, { 
        method: "POST", 
        headers: { "Content-Type": "application/json" }, 
        body: JSON.stringify({ userId, message }) 
      });
      
      const data = await response.json();
      console.log("User:", message);
      console.log("AI:", data.response);
      console.log("---");
    } catch (error) {
      console.error("Error:", error);
    }
  }
  
  // Test history endpoint
  try {
    const historyResponse = await fetch(`${API}/chat/${userId}/history`);
    const historyData = await historyResponse.json();
    console.log(`Total conversation history: ${historyData.total} messages`);
  } catch (error) {
    console.error("History error:", error);
  }
}

testClient().catch(console.error);