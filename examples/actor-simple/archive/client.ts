import { RivetClientClient } from "@rivet-gg/api";
import { ActorClient } from "../../sdks/actors/client/src/mod.ts"

// TODO: Get rid of this ugly garbage
const TOKEN = "env_svc.eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.CPy6y_yYQBD8ksXhtjIaEgoQ7vu2pTW9Sh-klx_juRpufSIXmgEUChIKEOaWqUrqXkn7iZjUQS58f40.gmHt4P3wSbfDY7gbGxvEdp4xW-SzFSFABUoOJBlxrv9WnnE_vWhwYIbgHRmYHTsNrbYiWZaOWpt4PVi9AdNcAA";
const client = new RivetClientClient({
	environment: "http://localhost:8080",
	// TODO: Allow not providing token in dev
	token: TOKEN,
});


async function main() {
	const actorClient = new ActorClient(client);

	// Broadcast event
	let broadcastActor;
	{
		const mod = 3;
		broadcastActor = await actorClient.getOrCreate({ name: "counter" }).connect(mod);
		broadcastActor.on("directCount", (count: unknown) => {
			console.log(`Direct (n % ${mod}):`, count);
		});
	}

	// Direct event
	let directActor;
	{
		const mod = 3;
		directActor = await actorClient.getOrCreate({ name: "counter" }).connect(mod);
		directActor.on("directCount", (count: unknown) => {
			console.log(`Broadcast:`, count);
		});
	}

	// Simple RPC
	{
		const newCount: number = await actorClient.getOrCreate({ name: "counter" }).rpc("increment", 5);
		console.log('Simple RPC:', newCount);
	}

	// Multiple RPC calls
	{
		const actor = actorClient.getOrCreate({ name: "counter" });

		for (let i = 0; i < 10; i++) {
			const output = await actor.rpc("increment", 5);
			console.log('Reusing handle:', output);
		}
	}

	// WebSocket
	{
		const actor = await actorClient.getOrCreate({ name: "counter" }).connect();
		for (let i = 0; i < 10; i++) {
			const newOutput = await actor.rpc("increment", 5);
			console.log('WebSocket:', newOutput);
		}

		actor.disconnect();
	}

	// Disconnect all actors before 
	broadcastActor.disconnect();
	directActor.disconnect();

	// Destroy
	{
		const actor = await actorClient.getOrCreate({ name: "counter" }).connect();
		await actor.destroy();
	}
}

await main();
