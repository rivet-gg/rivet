import { Cache, Context, Duration, Effect, Exit, Layer, Option, Schema } from "effect";
import { HttpClient, type HttpClientError, HttpClientResponse } from "effect/http";
import { npmEnv, run } from "../exec.ts";
import { unpackedSize } from "./tarball.ts";

const VersionManifest = Schema.Struct({
	dist: Schema.Struct({
		tarball: Schema.String,
		unpackedSize: Schema.optionalKey(Schema.Number),
	}),
});

const Packument = Schema.Struct({
	"dist-tags": Schema.Record(Schema.String, Schema.String),
});

type RegistryError = HttpClientError.HttpClientError | Schema.SchemaError;

export interface Interface {
	/** Base URL of the registry npm itself uses, so `.npmrc` and `npm_config_registry` apply. */
	readonly url: string;
	/**
	 * Unpacked size of `name@version`. Lookups are cached, so the same package
	 * across the latest, base, and PR sides costs one request.
	 */
	readonly packageSize: (name: string, version: string) => Effect.Effect<number, RegistryError>;
	/** The `latest` version of a package, or none if it isn't published. Cached like `packageSize`. */
	readonly latestVersion: (name: string) => Effect.Effect<Option.Option<string>, RegistryError>;
}

export class Registry extends Context.Service<Registry, Interface>()("pipboy/Registry") {}

const encodeName = (name: string) => name.replace("/", "%2F");

/** Cache successes for the whole run; drop failures so the next caller retries. */
const keepSuccesses = (exit: Exit.Exit<unknown, unknown>) => (Exit.isSuccess(exit) ? Duration.infinity : Duration.zero);

export const layer = Layer.effect(
	Registry,
	Effect.gen(function* () {
		const configured = (yield* run("npm", ["config", "get", "registry"], { env: npmEnv() })).trim();
		const url = configured.endsWith("/") ? configured : `${configured}/`;
		const http = (yield* HttpClient.HttpClient).pipe(HttpClient.retryTransient({ times: 4 }));

		// Old packages (published with npm < 5) have no `unpackedSize`; their tarball is measured instead.
		const lookupSize = Effect.fn("Registry.packageSize")(function* (id: string) {
			const at = id.lastIndexOf("@");
			const manifest = yield* http
				.get(`${url}${encodeName(id.slice(0, at))}/${id.slice(at + 1)}`)
				.pipe(Effect.flatMap(HttpClientResponse.filterStatusOk), Effect.flatMap(HttpClientResponse.schemaBodyJson(VersionManifest)));
			if (manifest.dist.unpackedSize !== undefined) return manifest.dist.unpackedSize;
			const tarball = yield* http.get(manifest.dist.tarball).pipe(Effect.flatMap(HttpClientResponse.filterStatusOk));
			return unpackedSize(new Uint8Array(yield* tarball.arrayBuffer));
		});
		const sizes = yield* Cache.makeWith(lookupSize, { capacity: 20_000, timeToLive: keepSuccesses });

		const lookupLatest = Effect.fn("Registry.latestVersion")(function* (name: string) {
			const response = yield* http.get(`${url}${encodeName(name)}`);
			if (response.status === 404) return Option.none<string>();
			const packument = yield* HttpClientResponse.filterStatusOk(response).pipe(
				Effect.flatMap(HttpClientResponse.schemaBodyJson(Packument)),
			);
			return Option.fromNullishOr(packument["dist-tags"].latest);
		});
		const latest = yield* Cache.makeWith(lookupLatest, { capacity: 1_000, timeToLive: keepSuccesses });

		return Registry.of({
			url,
			packageSize: (name, version) => Cache.get(sizes, `${name}@${version}`),
			latestVersion: (name) => Cache.get(latest, name),
		});
	}),
);
