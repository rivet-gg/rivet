import { RivetClient } from "@rivet-gg/api";
import readline from 'readline';

// Can be opt since they're not required for dev
const RIVET_ENDPOINT = process.env.RIVET_ENDPOINT;
const RIVET_SERVICE_TOKEN = process.env.RIVET_SERVICE_TOKEN;
const RIVET_PROJECT = process.env.RIVET_PROJECT;
const RIVET_ENVIRONMENT = process.env.RIVET_ENVIRONMENT;

let region = process.env.REGION;
if (!region || region.length === 0) {
	region = undefined;
}

const client = new RivetClient({
	environment: RIVET_ENDPOINT,
	token: RIVET_SERVICE_TOKEN,
});

let actorId;

async function run() {
	try {
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
		if (!port) throw new Error("missing port http");
		const actorOrigin = `${port.protocol}://${port.hostname}:${port.port}${port.path ?? ""}`;
		console.log("Created actor at", actorOrigin);

		// Setup readline interface for user input
		const rl = readline.createInterface({
			input: process.stdin,
			output: process.stdout
		});

		// Wait for user to press Enter
		await new Promise(resolve => {
			rl.question('Press Enter to destroy the actor...', () => {
				rl.close();
				resolve();
			});
		});
		
		if (actorId) {
			console.log("Destroying", actorId);
			await client.actor.destroy(actorId, {
				project: RIVET_PROJECT,
				environment: RIVET_ENVIRONMENT,
			});
		}
	} catch (error) {
		console.error("Error:", error);
		process.exit(1);
	}
}

run();
