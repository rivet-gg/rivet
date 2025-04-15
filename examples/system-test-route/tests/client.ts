import { RivetClient } from "@rivet-gg/api-full";
import crypto from "crypto";

// Can be opt since they're not required for dev
const RIVET_ENDPOINT = process.env.RIVET_ENDPOINT;
const RIVET_SERVICE_TOKEN = process.env.RIVET_SERVICE_TOKEN;
const RIVET_PROJECT = process.env.RIVET_PROJECT;
const RIVET_ENVIRONMENT = process.env.RIVET_ENVIRONMENT;

// Determine test kind from environment variable
const BUILD_NAME = process.env.BUILD;
if (BUILD_NAME !== "http-isolate" && BUILD_NAME !== "http-container") {
	throw new Error(
		"Must specify BUILD environment variable as either 'http-isolate' or 'http-container'",
	);
}

let region = process.env.REGION;
if (!region || region.length === 0) {
	region = undefined;
}

const client = new RivetClient({
	environment: RIVET_ENDPOINT,
	token: RIVET_SERVICE_TOKEN,
});

// Generate a random selector tag that will be used for both actors
const randomSelector = `test-${crypto.randomBytes(4).toString("hex")}`;
const routeNameId = `route-${crypto.randomBytes(4).toString("hex")}`;
const hostname = `${routeNameId}.rivet-job.local`;

async function run() {
	// Store info for cleanup
	const createdActorIds: string[] = [];
	let routeId: string | undefined;
	const actorIds = new Set<string>();
	let successfulMatches = 0;

	try {
		// Create actors with the random selector tag
		const numberOfActors = 0;

		for (let i = 1; i <= numberOfActors; i++) {
			console.time(`create actor ${i}`);
			console.log(`Creating actor ${i} with tag`, {
				selector: randomSelector,
			});

			const { actor } = await client.actors.create({
				project: RIVET_PROJECT,
				environment: RIVET_ENVIRONMENT,
				body: {
					region,
					tags: {
						selector: randomSelector,
						instance: i.toString(),
					},
					buildTags: { name: BUILD_NAME, current: "true" },
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
					...(BUILD_NAME === "http-container"
						? {
								resources: {
									cpu: 100,
									memory: 100,
								},
							}
						: {}),
				},
			});

			createdActorIds.push(actor.id);
			console.timeEnd(`create actor ${i}`);
			console.log(`Created actor ${i} with ID:`, actor.id);
		}

		// Wait for actors to be ready (brief delay)
		await new Promise((resolve) => setTimeout(resolve, 2000));

		// 3. Create a route with the random selector
		console.time(`create route`);
		console.log("Creating route with selector tag", {
			selector: randomSelector,
		});
		await client.routes.update(routeNameId, {
			project: RIVET_PROJECT,
			environment: RIVET_ENVIRONMENT,
			body: {
				hostname: hostname,
				path: "/test",
				routeSubpaths: true,
				selectorTags: {
					selector: randomSelector,
				},
			},
		});
		console.timeEnd(`create route`);
		console.log("Created route.");

		// Wait for route to be active
		await new Promise((resolve) => setTimeout(resolve, 2000));

		// 3.5. List routes and validate our route exists
		console.time("list routes");
		console.log("Listing routes to verify our route exists");
		const { routes } = await client.routes.list({
			project: RIVET_PROJECT,
			environment: RIVET_ENVIRONMENT,
		});
		console.timeEnd("list routes");
		
		// Find our route in the list
		const ourRoute = routes.find(route => route.hostname === hostname);
		if (!ourRoute) {
			throw new Error(`Route with hostname ${hostname} not found in routes list!`);
		}
		console.log("✅ Found our route in the list:", {
			routeId: ourRoute.id,
			hostname: ourRoute.hostname,
			path: ourRoute.path,
			routeSubpaths: ourRoute.routeSubpaths,
			selectorTags: ourRoute.selectorTags,
		});
		
		// Set the routeId for cleanup later
		routeId = ourRoute.id;

		// 4. Test making requests to the route until we get responses from both actors
		// Using localhost with Host header for local testing
		const testUrl = "http://localhost:7080";
		console.log(`Testing route at: ${testUrl} (with Host: ${hostname})`);
		console.time("route-test");

		let totalRequests = 0;
		const maxRequests = 20; // Maximum number of requests to try

		while (actorIds.size < 2 && totalRequests < maxRequests) {
			totalRequests++;

			try {
				// Make request to localhost but fake the hostname with the Host header
				const response = await fetch(testUrl, {
					headers: {
						Accept: "application/json",
						Host: hostname,
					},
				});

				if (!response.ok) {
					console.error(
						`Request failed: ${response.status} ${response.statusText}`,
					);
					continue;
				}

				const data = await response.json();
				console.log(
					`Request ${totalRequests}: Response from actor:`,
					data.actorId,
				);

				// Track the actor IDs we've seen
				if (data.actorId) {
					actorIds.add(data.actorId);
					successfulMatches++;
				}

				// If we've found both actors, we're done
				if (actorIds.size === 2) {
					console.log(
						"Successfully received responses from both actors!",
					);
					break;
				}

				// Small delay between requests
				await new Promise((resolve) => setTimeout(resolve, 200));
			} catch (error) {
				console.error("Error making request:", error);
				// Wait a bit longer if there's an error
				await new Promise((resolve) => setTimeout(resolve, 500));
			}
		}

		console.timeEnd("route-test");
		console.log(
			`Test completed. Matched ${actorIds.size}/2 actors in ${totalRequests} requests.`,
		);
		console.log(`Actors matched: ${Array.from(actorIds).join(", ")}`);

		if (actorIds.size < 2) {
			console.error("Failed to reach both actors through the route!");
		}

		// Final stats
		console.log(`
Route Test Results:
------------------
Total requests: ${totalRequests}
Successful responses: ${successfulMatches}
Unique actors reached: ${actorIds.size}/2
Route: ${testUrl} (Host: ${hostname})
Selector tag: ${randomSelector}
------------------
		`);
	} catch (error) {
		console.error("Error:", error);
	} finally {
		// Cleanup: delete route first
		if (routeId) {
			console.log("Deleting route", routeId);
			try {
				await client.routes.deleteRoute({
					project: RIVET_PROJECT,
					environment: RIVET_ENVIRONMENT,
					routeId,
				});
				console.log("Route deleted successfully");
			} catch (err) {
				console.error("Error deleting route:", err);
			}
		}

		// Then delete all actors
		for (let i = 0; i < createdActorIds.length; i++) {
			const actorId = createdActorIds[i];
			console.log(`Destroying actor ${i + 1}:`, actorId);
			try {
				await client.actors.destroy(actorId, {
					project: RIVET_PROJECT,
					environment: RIVET_ENVIRONMENT,
				});
			} catch (err) {
				console.error(`Error destroying actor ${i + 1}:`, err);
			}
		}
	}
}

// Run the test
run().catch(console.error);
