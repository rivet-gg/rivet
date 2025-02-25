import { Hono } from "hono";
import { logger } from "hono/logger";
import { serve } from "@hono/node-server";

// Validate required environment variables
if (!process.env.BETTERSTACK_TOKEN || !process.env.BETTERSTACK_HOST) {
  console.error("Error: BETTERSTACK_TOKEN and BETTERSTACK_HOST environment variables must be set");
  process.exit(1);
}

const app = new Hono();
app.use("*", logger());

app.get("/", (c) => c.text("Hello, world!"));

const port = parseInt(process.env.PORT_HTTP || "8080");
console.log(`Server starting on port ${port}`);

// Use the Node.js server adapter
serve({
  fetch: app.fetch,
  port: port
});
