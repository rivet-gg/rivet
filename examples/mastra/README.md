# Mastra Integration

This example demonstrates how to integrate Rivet with Mastra for AI agents with persistent state.

## Features

- **Persistent AI conversations** that survive server restarts
- **Real-time weather data** using Open-Meteo API
- **Memory system** for saving and recalling information
- **Per-user state isolation** with automatic persistence
- **Web interface** for testing and interaction

## Getting Started

### Prerequisites

- Node.js 18+
- OpenAI API key

### Environment Variables

Create a `.env` file with your OpenAI API key:

```bash
OPENAI_API_KEY=your-openai-api-key-here
```

### Development

Install dependencies and start the development server:

```bash
npm install
npm run dev
```

The server will start at http://localhost:8080 with a web interface for testing.

### Testing

Test the API endpoints directly:

```bash
# Send a message
curl -X POST http://localhost:8080/chat \
  -H "Content-Type: application/json" \
  -d '{"userId": "test-user", "message": "What'\''s the weather in Tokyo?"}'

# Get conversation history
curl http://localhost:8080/chat/test-user/history

# Clear conversation
curl -X DELETE http://localhost:8080/chat/test-user
```

## API Reference

### Chat Endpoint
```
POST /chat
Body: { "userId": string, "message": string }
Response: { "response": string, "messageId": string, "timestamp": number }
```

### History Endpoint
```
GET /chat/:userId/history
Response: { "history": Message[], "total": number, "actorName": string }
```

### Clear Endpoint
```
DELETE /chat/:userId
Response: { "success": boolean, "message": string }
```

## How It Works

The integration combines Rivet Actors for state persistence with Mastra agents for AI processing:

1. **Rivet Actors** store conversation history, user memory, and tool data
2. **Mastra Agents** process messages using OpenAI with access to tools
3. **Tools** can call external APIs (weather) and modify persistent state
4. **State automatically persists** across server restarts and user sessions

Each user gets their own isolated actor instance that maintains state between interactions.

## Architecture

```
┌─────────────────────────────────┐
│          Rivet Actor            │
│  ┌───────────────────────────┐  │
│  │      Mastra Agent         │  │
│  │   • OpenAI GPT-4o-mini    │  │
│  │   • Weather Tool          │  │
│  │   • Memory Tool           │  │
│  │   • Recall Tool           │  │
│  └───────────────────────────┘  │
│                                 │
│  Persistent State:              │
│  • messages[]                   │
│  • userMemory{}                 │
│  • toolData{}                   │
└─────────────────────────────────┘
```

## Example Interactions

**Weather Query:**
```
User: "What's the weather in Tokyo?"
AI: "The current weather in Tokyo is clear sky with a temperature of 18°C, feels like 16°C, humidity at 65%, and wind speed of 12 km/h."
```

**Memory System:**
```
User: "Remember my favorite color is blue"
AI: "I've remembered: my favorite color is blue"

User: "What do you remember about me?"
AI: "Here's what I remember:
- You told me: my favorite color is blue
- Last weather: Tokyo - Clear sky, 18°C"
```