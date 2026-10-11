import { spawn, type ChildProcess } from "node:child_process";
import { randomUUID } from "node:crypto";
import { createWriteStream } from "node:fs";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import getPort from "get-port";
import { afterAll, beforeAll, expect, test } from "vitest";
import { createClient } from "../src/client/mod";
import type { Registry } from "./fixtures/actor-streams-server";
import {
	getOrStartSharedTestEngine,
	releaseSharedTestEngine,
	TEST_ENGINE_TOKEN,
} from "./shared-engine";

const children: ChildProcess[] = [];
let endpoint: string;
let root: string;
let token: string;
let management: string;
let pool: string;
let startActorHost: (version: number) => ChildProcess;
let actorHost: ChildProcess;
let servicesHost: ChildProcess;
let startServicesHost: () => ChildProcess;

async function dsFetch(url: string, init: RequestInit = {}) {
	const headers = new Headers(init.headers);
	if (!headers.has("authorization"))
		headers.set("authorization", `Bearer ${token}`);
	return fetch(url, { ...init, headers });
}

async function until(check: () => Promise<boolean>, timeout = 30_000) {
	const end = Date.now() + timeout;
	while (Date.now() < end) {
		if (await check()) return;
		await new Promise((r) => setTimeout(r, 100));
	}
	throw new Error("Actor Streams condition timed out");
}

async function api(
	path: string,
	body?: unknown,
	credential = TEST_ENGINE_TOKEN,
): Promise<any> {
	const response = await fetch(endpoint + path, {
		method: body === undefined ? "GET" : "POST",
		headers: {
			authorization: `Bearer ${credential}`,
			"content-type": "application/json",
		},
		body: body === undefined ? undefined : JSON.stringify(body),
	});
	expect(response.ok, `Engine request: ${response.status}`).toBe(true);
	return response.json();
}

async function subscription(actorId: string, name = "primary") {
	const response = await dsFetch(
		`${root}/__ds/subscriptions/actor-${actorId}${name === "primary" ? "" : `-${name}`}`,
		{
			headers: { authorization: `Bearer ${management}` },
		},
	);
	if (response.status === 404 || response.status === 410) return undefined;
	expect(
		response.ok,
		`Subscription response: ${response.status} ${response.ok ? "" : await response.text()}`,
	).toBe(true);
	return response.json();
}

async function acknowledged(actorId: string) {
	await until(async () => {
		const sub = await subscription(actorId);
		if (!sub || sub.streams.length !== 1) return false;
		const stream = await dsFetch(`${root}/${sub.streams[0].path}`, {
			method: "HEAD",
		});
		expect(stream.ok).toBe(true);
		return (
			sub.streams[0].acked_offset ===
			stream.headers.get("stream-next-offset")
		);
	}, 90_000);
}

async function asleep(actorId: string) {
	await until(async () => {
		const result = await api(
			`/actors?namespace=default&actor_ids=${encodeURIComponent(actorId)}`,
		);
		return !!result.actors[0]?.sleep_ts;
	});
}

beforeAll(async () => {
	const engineBinaryPath = process.env.RIVET_STREAMS_TEST_ENGINE_BINARY;
	const servicesBinary = process.env.RIVET_STREAMS_TEST_SERVICES_BINARY;
	if (!engineBinaryPath || !servicesBinary)
		throw new Error(
			"Set RIVET_STREAMS_TEST_ENGINE_BINARY (signing-capable) and RIVET_STREAMS_TEST_SERVICES_BINARY (DS subscriptions + gateway auth).",
		);
	({ endpoint } = await getOrStartSharedTestEngine({
		gateway3: true,
		engineBinaryPath,
		deterministicScheduling: true,
	}));
	await until(
		async () =>
			(
				await fetch(`${endpoint}/namespaces?name=default`, {
					headers: { authorization: `Bearer ${TEST_ENGINE_TOKEN}` },
				})
			).ok,
	);
	const namespace = (await api("/namespaces?name=default")).namespaces[0]
		.namespace_id;
	const role = `actor-streams-${randomUUID()}`;
	const rules = Object.fromEntries(
		[
			["application_token", ["create"], { id: namespace }],
			["webhook_signing", ["create"], { id: namespace }],
			["namespace", ["read"], "any"],
			["datacenter", ["read", "list"], "any"],
			...[
				"actor",
				"runner",
				"runner_config",
				"actor_gateway",
				"actor_kv",
				"actor_stream",
				"durable_stream",
				"jwt",
			].map((resource) => [
				resource,
				[
					"read",
					"list",
					"create",
					"update",
					"delete",
					"append",
					"close",
				],
				{ id: namespace },
			]),
		].map(([resource, operations, ns]) => [
			`${role}-${resource}`,
			{ namespace: ns, resource, target: "any", operations },
		]),
	);
	await api("/acl/manifest", {
		rules,
		policies: { [role]: { rules: Object.keys(rules) } },
		roles: { [role]: { policies: [role] } },
	});
	token = (await api("/acl/tokens", { roles: [role], duration: 3600 })).token;
	management = randomUUID();
	pool = `streams-${randomUUID()}`;
	const port = await getPort();
	root = `http://127.0.0.1:${port}/durable-streams/v1/stream`;
	const logs = await mkdtemp(join(tmpdir(), "rivetkit-actor-streams-"));
	console.log(`Actor Streams logs: ${logs}`);
	const env = {
		...process.env,
		RIVET_ENDPOINT: endpoint,
		RIVET_NAMESPACE: "default",
		RIVET_TOKEN: token,
		RIVET_ENGINE_BINARY_PATH: engineBinaryPath,
		RIVET_ENGINE_BINARY: engineBinaryPath,
		RIVET_RUN_SERVICES: "0",
		RIVETKIT_ENGINE_SPAWN: "never",
		RUST_LOG: "warn",
		RIVET_LOG_LEVEL: "warn",
		DS_SUBSCRIPTIONS_MANAGEMENT_TOKEN: management,
		DS_SUBSCRIPTIONS_ALLOW_LOCAL_HTTP: "1",
		DS_ACTOR_ROUTING: "1",
		PUBLIC_ORIGIN: `http://127.0.0.1:${port}`,
		HOST: "0.0.0.0",
		PORT: String(port),
		RIVET_STREAMS_TEST_ROOT: root,
		RIVET_STREAMS_TEST_POOL: pool,
	};
	function start(
		command: string,
		args: string[],
		name: string,
		extraEnv: Record<string, string> = {},
	) {
		const child = spawn(command, args, {
			env: { ...env, ...extraEnv },
			stdio: ["ignore", "pipe", "pipe"],
		});
		const log = createWriteStream(join(logs, name));
		child.stdout!.pipe(log, { end: false });
		child.stderr!.pipe(log, { end: false });
		child.once("close", () => log.end());
		children.push(child);
		return child;
	}
	startServicesHost = () =>
		start(servicesBinary, ["start"], `services-${Date.now()}.log`);
	servicesHost = startServicesHost();
	startActorHost = (version) =>
		start(
			process.execPath,
			["--import", "tsx", "tests/fixtures/actor-streams-server.ts"],
			`actors-v${version}.log`,
			{ RIVET_STREAMS_TEST_VERSION: String(version) },
		);
	actorHost = startActorHost(1);
	await until(async () => {
		for (const child of children)
			if (child.exitCode !== null)
				throw new Error(`Host exited; inspect ${logs}`);
		const result = await api("/envoys?namespace=default");
		return (
			result.envoys.some((e: any) => e.pool_name === pool) &&
			result.envoys.some((e: any) => e.pool_name === "services")
		);
	}, 60_000);
}, 120_000);

afterAll(async () => {
	for (const child of children.reverse()) {
		child.kill("SIGTERM");
		await Promise.race([
			new Promise<void>((r) => child.once("exit", () => r())),
			new Promise<void>((r) =>
				setTimeout(() => {
					child.kill("SIGKILL");
					r();
				}, 3000),
			),
		]);
	}
	await releaseSharedTestEngine();
});

test("native onMessage: acceptance, signed gateway wake from sleep, ordered progress across restarts, and schema inference", async () => {
	const client = createClient<Registry>({
		endpoint,
		namespace: "default",
		token,
		poolName: pool,
	});
	const worker = client.worker.getOrCreate([randomUUID()]);
	await worker.stream.append({ id: 1 });
	const actorId = await worker.resolve();
	await acknowledged(actorId);
	expect(await worker.snapshot()).toEqual([1]);
	await asleep(actorId);

	// Append directly to DS: only its webhook can wake the sleeping application actor.
	const response = await dsFetch(`${root}/actors/worker/${actorId}`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify([{ id: 2 }, { id: 3, delay: 12_000 }]),
	});
	expect(response.ok).toBe(true);
	const tail = response.headers.get("stream-next-offset");
	await until(
		async () =>
			(await subscription(actorId)).streams[0].acked_offset === tail,
		60_000,
	);
	expect(await worker.snapshot()).toEqual([1, 2, 3]);
	await asleep(actorId);
	await worker.stream.append({ id: 4 });
	await acknowledged(actorId);
	expect(await worker.snapshot()).toEqual([1, 2, 3, 4]);
	await worker.appendSelf(5);
	await acknowledged(actorId);
	expect(await worker.snapshot()).toEqual([1, 2, 3, 4, 5]);

	const invalid = await fetch(
		`${endpoint}/gateway/${actorId}/internal/streams/wake`,
		{
			method: "POST",
			headers: {
				"x-rivet-token": token,
				"content-type": "application/json",
			},
			body: "{}",
		},
	);
	expect(invalid.status).toBe(401);
	const unauthorized = await fetch(
		`${endpoint}/gateway/${actorId}/internal/streams/wake`,
		{ method: "POST", body: "{}" },
	);
	expect(unauthorized.ok).toBe(false);

	const validated = client.validated.getOrCreate([randomUUID()]);
	await validated.stream.append({ id: "6" });
	await acknowledged(await validated.resolve());
	expect(await validated.snapshot()).toEqual([6]);
}, 180_000);

test("handler retries persist across restart, pause after five retries, and resume the blocked record", async () => {
	const client = createClient<Registry>({
		endpoint,
		namespace: "default",
		token,
		poolName: pool,
	});
	const worker = client.recovering.getOrCreate([randomUUID()]);
	await worker.stream.append({ id: 0 });
	const actorId = await worker.resolve();
	await acknowledged(actorId);
	const previous = (await subscription(actorId)).streams[0].acked_offset;
	await asleep(actorId);
	const response = await dsFetch(`${root}/actors/recovering/${actorId}`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify([{ id: 7 }, { id: 8 }, { id: 9 }]),
	});
	expect(response.ok).toBe(true);

	await until(async () => !!(await worker.stream.status()).failure);
	expect((await worker.snapshot()).messages).toEqual([0, 7]);
	const firstFailure = (await worker.stream.status()).failure!;
	expect(firstFailure.attempts).toBeLessThan(6);
	const old = actorHost;
	actorHost = startActorHost(1);
	await new Promise((r) => setTimeout(r, 1_000));
	old.kill("SIGTERM");
	await until(async () => {
		try {
			return (await worker.stream.status()).failure?.paused === true;
		} catch (error) {
			if (error instanceof Error && error.message.includes("HTTP 503"))
				return false;
			throw error;
		}
	}, 120_000);
	const paused = await worker.stream.status();
	expect(paused.failure?.attempts).toBe(6);
	expect(paused.failure?.firstFailedAt).toBe(firstFailure.firstFailedAt);
	expect((await subscription(actorId)).streams[0].acked_offset).toBe(
		previous,
	);
	await until(async () => (await worker.snapshot()).errors === 1);
	await asleep(actorId);
	expect((await worker.stream.status()).failure?.paused).toBe(true);
	const inputToken = await worker.issueToken();
	const denied = await fetch(
		`${endpoint}/gateway/${actorId}/internal/streams/resume`,
		{
			method: "POST",
			headers: {
				"x-rivet-token": inputToken.token,
				"content-type": "application/json",
			},
			body: "{}",
		},
	);
	expect(denied.status).toBe(403);
	await worker.unblock();
	await worker.stream.resume();
	await acknowledged(actorId);
	expect((await worker.snapshot()).messages).toEqual([0, 7, 8, 9]);
	expect((await worker.stream.status()).failure).toBeNull();
}, 240_000);

async function writeSource(path: string, id: string, method = "PUT") {
	const response = await dsFetch(`${root}/${path}`, {
		method,
		headers: { "content-type": "application/json" },
		body: JSON.stringify([{ id }]),
	});
	expect(
		response.ok,
		`Stream write: ${response.status} ${response.ok ? "" : await response.text()}`,
	).toBe(true);
	return response.headers.get("stream-next-offset");
}
async function namedAcknowledged(
	actorId: string,
	name: string,
	path: string,
	tail: string | null,
) {
	await until(
		async () =>
			(await subscription(actorId, name))?.streams.some(
				(s: any) => s.path === path && s.acked_offset === tail,
			),
		90_000,
	);
}
async function deploy(version: number) {
	const previous = actorHost;
	actorHost = startActorHost(version);
	await until(async () => {
		const names = await api("/actors/names?namespace=default");
		return (
			names.names.customer?.metadata.streamSubscriptionSources?.[pool]
				?.version === version
		);
	}, 60_000);
	previous.kill("SIGTERM");
}

test("pausing one named source does not block another source for the same actor", async () => {
	const tenant = randomUUID(),
		key = randomUUID();
	const bad = `tenants/${tenant}/orders/${key}/bad`;
	const good = `tenants/${tenant}/orders/${key}/good`;
	await writeSource(bad, "poison");
	const client = createClient<Registry>({
		endpoint,
		namespace: "default",
		token,
		poolName: pool,
	});
	const customer = client.customer.getOrCreate([tenant, key]);
	const id = await customer.resolve();
	await until(
		async () =>
			(
				await customer.stream.status({
					subscription: "orders",
					stream: bad,
				})
			).failure?.paused === true,
		60_000,
	);
	const tail = await writeSource(good, "healthy");
	await namedAcknowledged(id, "orders", good, tail);
	expect(await customer.snapshot()).toEqual(["orders:healthy"]);
	expect(
		(await customer.stream.status({ subscription: "orders", stream: bad }))
			.failure?.attempts,
	).toBe(6);
}, 150_000);

test("one paired token authorizes actor and direct DS append, but rejects other stream operations and invalid credentials", async () => {
	const admin = createClient<Registry>({
		endpoint,
		namespace: "default",
		token,
		poolName: pool,
	});
	const owner = admin.worker.getOrCreate([randomUUID()]);
	const actorId = await owner.resolve();
	const paired = await owner.issueToken({
		permissions: { actor_gateway: ["append"] },
	});
	const client = createClient<Registry>({
		endpoint,
		namespace: "default",
		token: paired.token,
		poolName: pool,
		disableMetadataLookup: true,
	});
	await client.worker.getForId(actorId).stream.append({ id: 1 });
	const url = `${root}/actors/worker/${actorId}`;
	const headers = {
		authorization: `Bearer ${paired.token}`,
		"content-type": "application/json",
	};
	expect(
		(
			await fetch(url, {
				method: "POST",
				headers,
				body: JSON.stringify([{ id: 2 }]),
			})
		).ok,
	).toBe(true);
	await acknowledged(actorId);
	for (const method of ["GET", "HEAD", "DELETE"]) {
		expect((await fetch(url, { method, headers })).status).toBe(403);
	}
	expect(
		(
			await fetch(url, {
				method: "POST",
				headers: { ...headers, "stream-closed": "true" },
			})
		).status,
	).toBe(403);
	expect(
		(
			await fetch(url, {
				method: "POST",
				headers: { ...headers, "stream-closed": "TRUE" },
			})
		).status,
	).toBe(403);
	expect((await fetch(url, { method: "POST", body: "[]" })).status).toBe(401);
	expect(
		(
			await fetch(url, {
				method: "POST",
				headers: { authorization: "Bearer invalid" },
				body: "[]",
			})
		).status,
	).toBe(401);
	const actionOnly = await owner.issueToken({
		permissions: { actor_gateway: ["read"] },
	});
	expect(
		(
			await fetch(url, {
				method: "POST",
				headers: {
					...headers,
					authorization: `Bearer ${actionOnly.token}`,
				},
				body: "[]",
			})
		).status,
	).toBe(403);
	const otherNamespace = `streams-${randomUUID()}`;
	await api("/namespaces", {
		name: otherNamespace,
		display_name: "Stream auth test",
	});
	const wrong = await api("/auth/tokens", {
		namespace: otherNamespace,
		grants: [
			{
				resource: "actor_stream",
				target: { id: actorId },
				operations: ["append"],
			},
		],
	});
	expect(
		(
			await fetch(url, {
				method: "POST",
				headers: { ...headers, authorization: `Bearer ${wrong.token}` },
				body: "[]",
			})
		).status,
	).toBe(403);
	const short = await owner.issueToken({
		expiresIn: 1,
		permissions: { actor_gateway: ["append"] },
	});
	await until(
		async () =>
			(
				await fetch(url, {
					method: "POST",
					headers: {
						...headers,
						authorization: `Bearer ${short.token}`,
					},
					body: "[]",
				})
			).status === 401,
		45_000,
	);
}, 90_000);

test("destroyed keyed destinations are replaced and destroyed primary IDs are not recreated", async () => {
	const tenant = randomUUID(),
		key = randomUUID();
	const path = `tenants/${tenant}/orders/${key}/replacement`;
	const tail = await writeSource(path, "before");
	const lookup = async () =>
		(
			await api(
				`/actors?namespace=default&name=customer&key=${encodeURIComponent(`${tenant}/${key}`)}`,
			)
		).actors[0]?.actor_id;
	let id = "";
	await until(async () => !!(id = await lookup()));
	await namedAcknowledged(id, "orders", path, tail);
	const admin = createClient<Registry>({
		endpoint,
		namespace: "default",
		token,
		poolName: pool,
	});
	const old = admin.customer.getForId(id);
	await old.stream.append({ id: "primary-before" });
	await acknowledged(id);
	const destroyed = await fetch(
		`${endpoint}/actors/${id}?namespace=default`,
		{ method: "DELETE", headers: { authorization: `Bearer ${token}` } },
	);
	expect(destroyed.ok).toBe(true);
	const next = await writeSource(path, "after", "POST");
	let replacement = "";
	await until(
		async () => !!(replacement = await lookup()) && replacement !== id,
		60_000,
	);
	await namedAcknowledged(replacement, "orders", path, next);
	expect(await admin.customer.getForId(replacement).snapshot()).toEqual([
		"orders:after",
	]);
	await writeSource(`actors/customer/${id}`, "primary-after", "POST");
	await until(async () => !(await subscription(id)), 60_000);
	expect(await lookup()).toBe(replacement);
}, 120_000);

test("named subscriptions route first writes, multiple sources, deployment additions and removals without waking actors to register", async () => {
	const client = createClient<Registry>({
		endpoint,
		namespace: "default",
		token,
		poolName: pool,
	});
	const tenant = randomUUID(),
		key = randomUUID();
	const first = `tenants/${tenant}/orders/${key}/one`;
	// No application actor exists yet. DS resolves the key and includes PUT's body.
	const firstTail = await writeSource(first, "first");
	let id = "";
	await until(async () => {
		const result = await api(
			`/actors?namespace=default&name=customer&key=${encodeURIComponent(`${tenant}/${key}`)}`,
		);
		id = result.actors[0]?.actor_id ?? "";
		return !!id;
	});
	const customer = client.customer.getForId(id);
	await namedAcknowledged(id, "orders", first, firstTail);
	expect(await customer.snapshot()).toEqual(["orders:first"]);
	await asleep(id);
	// A stream name cannot impersonate another actor type. Malformed ID paths
	// must also leave unrelated routing jobs able to make progress.
	await writeSource(`actors/worker/${id}`, "wrong-type");
	await writeSource("actors/customer/not-an-actor-id", "invalid-id");
	const second = `tenants/${tenant}/orders/${key}/two`;
	await namedAcknowledged(
		id,
		"orders",
		second,
		await writeSource(second, "second"),
	);
	expect(await customer.snapshot()).toEqual([
		"orders:first",
		"orders:second",
	]);
	expect(await subscription(id)).toBeUndefined();
	await customer.stream.append({ id: "input" });
	await acknowledged(id);
	await asleep(id);
	servicesHost.kill("SIGTERM");
	await Promise.race([
		new Promise<void>((resolve) =>
			servicesHost.once("exit", () => resolve()),
		),
		new Promise<void>((resolve) => setTimeout(resolve, 3_000)),
	]);
	if (servicesHost.exitCode === null) servicesHost.kill("SIGKILL");
	servicesHost = startServicesHost();
	await until(async () => {
		try {
			return (await dsFetch(`${root}/__ds/jwks.json`)).ok;
		} catch {
			return false;
		}
	}, 60_000);
	await namedAcknowledged(
		id,
		"orders",
		first,
		await writeSource(first, "after-restart", "POST"),
	);
	expect(await customer.snapshot()).toEqual([
		"orders:first",
		"orders:second",
		"primary:input",
		"orders:after-restart",
	]);
	await asleep(id);

	const added = `tenants/${tenant}/notifications/${key}`;
	await writeSource(added, "historical");
	await deploy(2);
	// DS adopts metadata on its polling interval; activation is unnecessary.
	await new Promise((r) => setTimeout(r, 6_000));
	await namedAcknowledged(
		id,
		"notifications",
		added,
		await writeSource(added, "new", "POST"),
	);
	expect(await customer.snapshot()).toEqual([
		"orders:first",
		"orders:second",
		"primary:input",
		"orders:after-restart",
		"notifications:new",
	]);
	await asleep(id);

	// A stale host reconnect must not roll the authoritative index back.
	const stale = startActorHost(1);
	await new Promise((r) => setTimeout(r, 2_000));
	const names = await api("/actors/names?namespace=default");
	expect(
		names.names.customer.metadata.streamSubscriptionSources[pool].version,
	).toBe(2);
	stale.kill("SIGTERM");

	await deploy(3);
	await until(
		async () => (await subscription(id, "orders"))?.streams.length === 0,
		30_000,
	);
	await writeSource(first, "removed", "POST");
	await namedAcknowledged(
		id,
		"notifications",
		added,
		await writeSource(added, "after-removal", "POST"),
	);
	expect(await customer.snapshot()).toEqual([
		"orders:first",
		"orders:second",
		"primary:input",
		"orders:after-restart",
		"notifications:new",
		"notifications:after-removal",
	]);
	await asleep(id);
	await deploy(4);
	await until(
		async () =>
			(await subscription(id, "notifications"))?.streams.length === 0,
		30_000,
	);
	const removed = await api("/actors/names?namespace=default");
	expect(
		removed.names.customer.metadata.streamSubscriptionSources[pool]
			.subscriptions,
	).toEqual({});
}, 240_000);

test("dead letters continue after durable handoff and redrive once at the primary tail", async () => {
	const client = createClient<Registry>({
		endpoint,
		namespace: "default",
		token,
		poolName: pool,
	});
	const worker = client.deadLetters.getOrCreate([`dlq-${randomUUID()}`]);
	const actorId = await worker.resolve();
	await worker.stream.append({ id: 7 });
	await worker.stream.append({ id: 8 });
	await worker.stream.append({ id: 9 });
	await until(
		async () => (await worker.snapshot()).messages.join(",") === "7,9",
		90_000,
	);
	await acknowledged(actorId);
	const status = await worker.stream.status();
	expect(status.failure).toBeNull();
	expect(status.deadLetter).toBeNull();
	expect((await worker.snapshot()).errors).toBe(1);
	const response = await dsFetch(
		`${root}/${status.deadLetterStream}?offset=-1`,
	);
	expect(response.ok).toBe(true);
	const letters = await response.json();
	expect(letters).toHaveLength(1);
	const letter = letters[0];
	expect(letter.body).toEqual({ id: 8 });
	expect(letter.failure.attempts).toBe(6);
	await worker.unblock();
	const paired = await worker.issueToken();
	const redrive = () =>
		fetch(`${endpoint}/gateway/${actorId}/internal/streams/redrive`, {
			method: "POST",
			headers: {
				"x-rivet-token": paired.token,
				"content-type": "application/json",
			},
			body: JSON.stringify({
				message: letter,
				idempotencyKey: "repair-1",
			}),
		});
	let first: Response;
	await until(async () => {
		first = await redrive();
		return first.status !== 409;
	});
	expect(first!.ok).toBe(true);
	const receipt = await first!.json();
	await until(
		async () => (await worker.snapshot()).messages.join(",") === "7,9,8",
	);
	await acknowledged(actorId);
	const repeat = await redrive();
	expect(repeat.ok).toBe(true);
	expect(await repeat.json()).toEqual(receipt);
	expect((await worker.snapshot()).messages).toEqual([7, 9, 8]);
	const actionOnly = await worker.issueToken({
		permissions: { actor_gateway: ["read"] },
	});
	const denied = await fetch(
		`${endpoint}/gateway/${actorId}/internal/streams/redrive`,
		{
			method: "POST",
			headers: {
				"x-rivet-token": actionOnly.token,
				"content-type": "application/json",
			},
			body: JSON.stringify({
				message: letter,
				idempotencyKey: "repair-2",
			}),
		},
	);
	expect(denied.status).toBe(403);
	const other = { ...letter, subscription: "orders" };
	await expect(
		worker.stream.redrive(other, { idempotencyKey: "shared" }),
	).rejects.toThrow();
	await expect(
		worker.stream.redrive(
			{ ...letter, body: { id: 10 } },
			{ idempotencyKey: "repair-1" },
		),
	).rejects.toThrow();
}, 180_000);
