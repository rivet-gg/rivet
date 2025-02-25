import { Hono } from "hono";
import { logger } from "hono/logger";
import { serve } from "@hono/node-server";

// Read environment variables
const BETTERSTACK_TOKEN = process.env.BETTERSTACK_TOKEN;
const BETTERSTACK_HOST = process.env.BETTERSTACK_HOST;

// Validate required environment variables in production
if (process.env.NODE_ENV === 'production') {
  if (!BETTERSTACK_TOKEN) {
    throw new Error("BETTERSTACK_TOKEN environment variable is required in production");
  }
  if (!BETTERSTACK_HOST) {
    throw new Error("BETTERSTACK_HOST environment variable is required in production");
  }
}

const app = new Hono();
app.use("*", logger());

app.get("/", (c) => c.text("Hello, world!"));
app.get("/health", (c) => c.json({ status: "ok" }));

// Add metrics endpoint
let requestCount = 0;
app.use("*", async (c, next) => {
  const start = Date.now();
  requestCount++;
  await next();
  const duration = Date.now() - start;
  console.log(JSON.stringify({
    type: "metric",
    name: "request_duration",
    value: duration,
    tags: { path: c.req.path }
  }));
});

app.get("/metrics", (c) => c.json({
  requests: requestCount
}));

const port = parseInt(process.env.PORT_HTTP || "8080");
console.log(`Server starting on port ${port}`);

// Use the Node.js server adapter
const server = serve({
  fetch: app.fetch,
  port: port
});

// Handle graceful shutdown
process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});
