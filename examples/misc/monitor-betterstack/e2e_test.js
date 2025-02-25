import { RivetClient } from "@rivet-gg/api";
import { assertExists } from "@std/assert";

// Can be opt since they're not required for dev
const RIVET_ENDPOINT = Deno.env.get("RIVET_ENDPOINT");
const RIVET_SERVICE_TOKEN = Deno.env.get("RIVET_SERVICE_TOKEN");
const RIVET_PROJECT = Deno.env.get("RIVET_PROJECT");
const RIVET_ENVIRONMENT = Deno.env.get("RIVET_ENVIRONMENT");

let region = Deno.env.get("REGION");
if (!region || region.length === 0) {
	region = undefined;
}

const client = new RivetClient({
	environment: RIVET_ENDPOINT,
	token: RIVET_SERVICE_TOKEN,
});

async function run() {
		console.log("Creating actor", { region });
		const { actor } = await client.actor.create({
			project: RIVET_PROJECT,
			environment: RIVET_ENVIRONMENT,
			body: {
				region,
				tags: {
					name: "example",
				},
				buildTags: { name: "example", current: "true" },
				network: {
					ports: {
						http: {
							protocol: "https",
							routing: {
								guard: {},
							},
						},
					},
				},
				lifecycle: {
					durable: false,
				},
			},
		});
		actorId = actor.id;

		const port = actor.network.ports.http;
		assertExists(port, "missing port http");
		const actorOrigin = `${port.protocol}://${port.hostname}:${port.port}${port.path ?? ""}`;
		console.log("Created actor at", actorOrigin);

		console.log("Press Enter to destroy the actor...");
		await new Promise(resolve => Deno.stdin.read(new Uint8Array(1)).then(resolve));
		
		if (actorId) {
			console.log("Destroying", actorId);
			await client.actor.destroy(actorId, {
				project: RIVET_PROJECT,
				environment: RIVET_ENVIRONMENT,
			});
		}
}

run();
