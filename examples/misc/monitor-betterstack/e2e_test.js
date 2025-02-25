import { RivetClient } from "@rivet-gg/api";
import readline from "readline";

// Read environment variables
const RIVET_ENDPOINT = process.env.RIVET_ENDPOINT;
const RIVET_SERVICE_TOKEN = process.env.RIVET_SERVICE_TOKEN;
const RIVET_PROJECT = process.env.RIVET_PROJECT;
const RIVET_ENVIRONMENT = process.env.RIVET_ENVIRONMENT;

// BetterStack credentials are now loaded from .env file in the Docker image

// Check required environment variables
if (!RIVET_SERVICE_TOKEN) {
	throw new Error("RIVET_SERVICE_TOKEN environment variable is required");
}
if (!RIVET_PROJECT) {
	throw new Error("RIVET_PROJECT environment variable is required");
}
if (!RIVET_ENVIRONMENT) {
	throw new Error("RIVET_ENVIRONMENT environment variable is required");
}

let region = process.env.REGION;
if (!region || region.length === 0) {
	region = undefined;
}

const client = new RivetClient({
	environment: RIVET_ENDPOINT,
	token: RIVET_SERVICE_TOKEN,
});

let actorId;

// Function to clean up and destroy the actor
async function cleanupActor() {
	if (actorId) {
		console.log("\nDestroying actor", actorId);
		try {
			await client.actor.destroy(actorId, {
				project: RIVET_PROJECT,
				environment: RIVET_ENVIRONMENT,
			});
			console.log("Actor destroyed successfully");
		} catch (error) {
			console.error("Error destroying actor:", error);
		}
		process.exit(0);
	}
}

// Handle Ctrl+C
process.on('SIGINT', async () => {
	console.log("\nCtrl+C detected, cleaning up...");
	await cleanupActor();
});

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
				resources: {
					cpu: 100,
					memory: 100,
				},
				// No need to pass BetterStack credentials as they're hardcoded in the Dockerfile
				env: {},
				lifecycle: {
					durable: false,
				},
			},
		});
		actorId = actor.id;

		const port = actor.network.ports.http;
		if (!port) throw new Error("missing port http");
		console.log("Created actor at", port.url);

		// Setup readline interface for user input
		const rl = readline.createInterface({
			input: process.stdin,
			output: process.stdout,
		});

		// Wait for user to press Enter
		await new Promise((resolve) => {
			rl.question("Press Enter to destroy the actor...", () => {
				rl.close();
				resolve();
			});
		});

		await cleanupActor();
	} catch (error) {
		console.error("Error:", error);
		process.exit(1);
	}
}

run();
