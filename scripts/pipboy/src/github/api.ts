import { Config, Context, Effect, Layer, Redacted, Schema } from "effect";
import { type HttpBody, HttpClient, type HttpClientError, HttpClientRequest, HttpClientResponse } from "effect/http";

/** GitHub Actions context, read from the standard runner environment. */
export const ActionsEnv = Config.all({
	token: Config.Redacted("GITHUB_TOKEN"),
	repository: Config.String("GITHUB_REPOSITORY"),
	runId: Config.String("GITHUB_RUN_ID"),
	runAttempt: Config.String("GITHUB_RUN_ATTEMPT").pipe(Config.withDefault("1")),
	apiUrl: Config.String("GITHUB_API_URL").pipe(Config.withDefault("https://api.github.com")),
	serverUrl: Config.String("GITHUB_SERVER_URL").pipe(Config.withDefault("https://github.com")),
});

export interface Interface {
	readonly repository: string;
	readonly runId: string;
	readonly runAttempt: string;
	readonly runUrl: string;
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

export const layer = Layer.effect(
	GitHub,
	Effect.gen(function* () {
		const env = yield* ActionsEnv;
		const http = (yield* HttpClient.HttpClient).pipe(
			HttpClient.mapRequest((request) =>
				request.pipe(
					HttpClientRequest.prependUrl(env.apiUrl),
					HttpClientRequest.bearerToken(Redacted.value(env.token)),
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
			repository: env.repository,
			runId: env.runId,
			runAttempt: env.runAttempt,
			runUrl: `${env.serverUrl}/${env.repository}/actions/runs/${env.runId}`,
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
