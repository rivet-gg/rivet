import { createClient } from "rivetkit/client";
import type { Registry } from "./index";

const client = createClient<Registry>({
	endpoint: process.env.RIVET_ENDPOINT!,
});
const account = client.account.getOrCreate(["customer-123"]);
await account.stream.append({ type: "credit", amount: 25 });
// Append confirms durable acceptance. Processing can finish later.
