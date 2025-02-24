import { Hono } from "hono";
import { logger } from "hono/logger";

const app = new Hono();
app.use("*", logger());

app.get("/", (c) => c.text("Hello Hono!"));

const port = process.env.PORT_HTTP || 8080;
console.log(`Server starting on port ${port}`);

app.listen(port);
