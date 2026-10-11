import { createClient } from "rivetkit/client";
import type { registry } from "../actors-streams/index";

const client = createClient<typeof registry>(process.env.RIVET_ENDPOINT!);
const customer = client.customer.getOrCreate(["acme", "alice"]);

// Actions plus input to this actor's primary stream, through either append path.
const inputToken = await customer.issueToken({
	subject: "alice",
	expiresIn: 60,
});

// Explicit permissions replace the default: this token only grants action access.
const actionToken = await customer.issueToken({
	subject: "alice",
	expiresIn: 60,
	permissions: { actor_gateway: ["read"] },
});

export { inputToken, actionToken };
