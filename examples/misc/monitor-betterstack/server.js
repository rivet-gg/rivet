import { Hono } from "hono";
import { logger } from "hono/logger";
import { serve } from "@hono/node-server";

const app = new Hono();
app.use("*", logger());

app.get("/", (c) => c.text("Hello, world!"));
app.get("/health", (c) => c.json({ status: "ok" }));

const port = parseInt(process.env.PORT_HTTP || "8080");
console.log(`Server starting on port ${port}`);

// Use the Node.js server adapter
serve({
  fetch: app.fetch,
  port: port
});
