import { createClient } from "rivetkit/client";
import type { registry } from "./index";

const client = createClient<typeof registry>(process.env.RIVET_ENDPOINT!);
const customer = client.customer.getOrCreate(["acme", "alice"]);

const primary = await customer.stream.status();
if (primary.failure?.paused) {
	// Fix the cause first. Resuming retries the same blocked record.
	await customer.stream.resume();
}

// Progress and failures are tracked separately for each subscribed source.
const source = {
	subscription: "orders",
	stream: "tenants/acme/orders/alice",
};
const orders = await customer.stream.status(source);
if (orders.failure?.paused) await customer.stream.resume(source);
