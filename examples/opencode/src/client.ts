import { createClient } from "rivetkit/client";
import type { registry } from "./registry.js";

const client = createClient<typeof registry>("http://localhost:6420");
const openCode = client.opencode.getOrCreate(["demo"]);

await openCode.prompt(
	process.argv.slice(2).join(" ") || "Summarize this project.",
);
await openCode.waitForIdle();
const messages = await openCode.getMessages();
console.log(messages);
