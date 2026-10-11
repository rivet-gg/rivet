import { createClient } from "rivetkit/client";
import type { registry } from "./index";

const client = createClient<typeof registry>(process.env.RIVET_ENDPOINT!);
const customer = client.customer.getOrCreate(["acme", "alice"]);

await customer.stream.append({ amount: 5 });
