import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { type Server, createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import { afterEach, beforeEach, describe, expect, test } from "vitest";

const run = promisify(execFile);
const CLI = join(import.meta.dirname, "../src/cli.ts");
const MB = 1024 ** 2;

interface Published {
	readonly manifest: Record<string, unknown>;
	readonly tarball: Buffer;
}

/** A minimal npm registry: packuments and tarballs over real HTTP, for the real npm client. */
class Registry {
	private readonly packages = new Map<string, Map<string, Published>>();
	private server: Server | undefined;
	url = "";

	async start() {
		this.server = createServer((req, res) => {
			const path = decodeURIComponent(req.url ?? "/").slice(1);
			const tarball = path.match(/^(.+)\/-\/(.+)\.tgz$/);
			if (tarball) {
				const [, name, file] = tarball;
				const version = file?.slice(file.lastIndexOf("-") + 1);
				const published = this.packages.get(name ?? "")?.get(version ?? "");
				if (!published) return res.writeHead(404).end();
				return res.writeHead(200, { "content-type": "application/octet-stream" }).end(published.tarball);
			}
			const [name, version] = path.includes("/") && !path.startsWith("@") ? path.split("/") : [path, undefined];
			const versions = this.packages.get(name ?? "");
			if (!versions) return res.writeHead(404).end("{}");
			const body =
				version === undefined
					? {
							name,
							"dist-tags": { latest: [...versions.keys()].at(-1) },
							versions: Object.fromEntries([...versions].map(([v, p]) => [v, p.manifest])),
						}
					: versions.get(version)?.manifest;
			res.writeHead(body ? 200 : 404, { "content-type": "application/json" }).end(JSON.stringify(body ?? {}));
		});
		await new Promise<void>((resolve) => this.server?.listen(0, "127.0.0.1", resolve));
		this.url = `http://127.0.0.1:${(this.server.address() as AddressInfo).port}/`;
	}

	stop() {
		this.server?.close();
	}

	/** Publishes a package. `unpackedSize` is what the registry reports, which is all the check reads. */
	async publish(name: string, version: string, dependencies: Record<string, string>, unpackedSize: number) {
		const tarball = await packFixture(name, version, dependencies);
		const manifest = {
			name,
			version,
			dependencies,
			dist: {
				tarball: `${this.url}${name}/-/${name}-${version}.tgz`,
				shasum: createHash("sha1").update(tarball).digest("hex"),
				integrity: `sha512-${createHash("sha512").update(tarball).digest("base64")}`,
				unpackedSize,
			},
		};
		const versions = this.packages.get(name) ?? new Map<string, Published>();
		versions.set(version, { manifest, tarball });
		this.packages.set(name, versions);
	}
}

async function packFixture(name: string, version: string, dependencies: Record<string, string>): Promise<Buffer> {
	const dir = mkdtempSync(join(tmpdir(), "pipboy-fixture-"));
	writeFileSync(join(dir, "package.json"), JSON.stringify({ name, version, main: "index.js", dependencies }));
	writeFileSync(join(dir, "index.js"), `export const name = ${JSON.stringify(name)};\n`);
	await run("npm", ["pack", "--pack-destination", dir], { cwd: dir });
	const file = readdirSync(dir).find((f) => f.endsWith(".tgz")) ?? "";
	const tarball = readFileSync(join(dir, file));
	rmSync(dir, { recursive: true, force: true });
	return tarball;
}

/** A `pack` output directory holding one workspace package, as CI produces for a commit. */
async function packedSide(dependencies: Record<string, string>): Promise<string> {
	const dir = mkdtempSync(join(tmpdir(), "pipboy-side-"));
	const file = "rivetkit-0.0.0-pipboy.tgz";
	writeFileSync(join(dir, file), await packFixture("rivetkit", "0.0.0-pipboy", dependencies));
	writeFileSync(join(dir, "manifest.json"), JSON.stringify({ version: "0.0.0-pipboy", tarballs: { rivetkit: file } }));
	return dir;
}

async function footprint(registry: Registry, pr: string, base: string) {
	const out = join(mkdtempSync(join(tmpdir(), "pipboy-out-")), "footprint.json");
	const env = {
		...process.env,
		npm_config_registry: registry.url,
		npm_config_cache: join(tmpdir(), `pipboy-npm-cache-${process.pid}`),
	};
	try {
		const { stdout } = await run(process.execPath, ["--import", "tsx", CLI, "footprint", "--pr", pr, "--base", base, "--out", out], { env });
		return { exitCode: 0, output: stdout };
	} catch (error) {
		const e = error as { code: number; stdout: string; stderr: string };
		return { exitCode: e.code, output: `${e.stdout}\n${e.stderr}` };
	}
}

describe("footprint", () => {
	// A fresh registry per test, so each test publishes exactly what it needs.
	let registry: Registry;
	beforeEach(async () => {
		registry = new Registry();
		await registry.start();
	});
	afterEach(() => registry.stop());

	test("fails when the PR itself pushes a package over its budget, and names the dependency", async () => {
		await registry.publish("dep", "0.2.1", {}, 1 * MB);
		await registry.publish("heavy", "1.0.0", {}, 600 * MB);
		await registry.publish("rivetkit", "1.0.0", { dep: "^0.2.1" }, 5 * MB);

		const base = await packedSide({ dep: "^0.2.1" });
		const pr = await packedSide({ dep: "^0.2.1", heavy: "1.0.0" });
		const result = await footprint(registry, pr, base);

		expect(result.exitCode).not.toBe(0);
		expect(result.output).toContain("over its");
		expect(result.output).toContain("`heavy@1.0.0`");
		expect(result.output).toContain("`rivetkit › heavy`");
	}, 120_000);

	test("reports registry drift without failing an unchanged PR, and traces it to the floating 0.x range", async () => {
		await registry.publish("dep", "0.2.1", {}, 1 * MB);
		await registry.publish("heavy", "1.0.0", {}, 600 * MB);
		await registry.publish("rivetkit", "1.0.0", { dep: "^0.2.1" }, 5 * MB);
		// September 2026: a patch inside `^0.2.1` starts pulling in a huge package.
		await registry.publish("dep", "0.2.20", { heavy: "1.0.0" }, 1 * MB);

		const side = await packedSide({ dep: "^0.2.1" });
		const result = await footprint(registry, side, side);

		expect(result.exitCode).toBe(0);
		expect(result.output).toContain("Registry drift");
		expect(result.output).toContain("`rivetkit › dep › heavy`");
		expect(result.output).toContain("`dep@^0.2.1`");
		expect(result.output).toContain("`0.2.20`");
	}, 120_000);
});
