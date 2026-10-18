import { Config, Context, Effect, Layer, Redacted, Schema } from "effect";
import { type HttpBody, HttpClient, type HttpClientError, HttpClientRequest, HttpClientResponse } from "effect/http";

export interface Interface {
	readonly repository: string;
	readonly get: <S extends Schema.Constraint>(
		path: string,
		schema: S,
	) => Effect.Effect<S["Type"], HttpClientError.HttpClientError | Schema.SchemaError, S["DecodingServices"]>;
	readonly send: (
		method: "POST" | "PATCH",
		path: string,
		body: unknown,
	) => Effect.Effect<void, HttpClientError.HttpClientError | HttpBody.HttpBodyError>;
}

export class GitHub extends Context.Service<GitHub, Interface>()("pipboy/GitHub") {}

/** The current GitHub Actions run, read from the standard runner environment. */
export const CurrentRun = Config.all({
	repository: Config.String("GITHUB_REPOSITORY"),
	runId: Config.String("GITHUB_RUN_ID"),
	runAttempt: Config.String("GITHUB_RUN_ATTEMPT").pipe(Config.withDefault("1")),
	serverUrl: Config.String("GITHUB_SERVER_URL").pipe(Config.withDefault("https://github.com")),
}).pipe(Config.map((run) => ({ ...run, url: `${run.serverUrl}/${run.repository}/actions/runs/${run.runId}` })));

/** A GitHub API client for `repository`, authenticated with `token` or `GITHUB_TOKEN`. */
export const layerFor = (repository: string, explicitToken?: Redacted.Redacted<string>) =>
	Layer.effect(
		GitHub,
		Effect.gen(function* () {
			const token = explicitToken ?? (yield* Config.Redacted("GITHUB_TOKEN"));
			const apiUrl = yield* Config.String("GITHUB_API_URL").pipe(Config.withDefault("https://api.github.com"));
			const http = (yield* HttpClient.HttpClient).pipe(
				HttpClient.mapRequest((request) =>
					request.pipe(
						HttpClientRequest.prependUrl(apiUrl),
						HttpClientRequest.bearerToken(Redacted.value(token)),
						HttpClientRequest.setHeader("accept", "application/vnd.github+json"),
						HttpClientRequest.setHeader("x-github-api-version", "2022-11-28"),
						// GitHub rejects API requests without a User-Agent.
						HttpClientRequest.setHeader("user-agent", "rivet-pipboy"),
					),
				),
				HttpClient.filterStatusOk,
				HttpClient.retryTransient({ times: 4 }),
			);
			return GitHub.of({
				repository,
				get: (path, schema) => http.get(path).pipe(Effect.flatMap(HttpClientResponse.schemaBodyJson(schema))),
				send: (method, path, body) =>
					HttpClientRequest.make(method)(path).pipe(
						HttpClientRequest.bodyJson(body),
						Effect.flatMap(http.execute),
						Effect.asVoid,
					),
			});
		}),
	);

/** A client for the repository of the current Actions run. */
export const layer = Layer.unwrap(Config.String("GITHUB_REPOSITORY").pipe(Effect.map(layerFor)));
