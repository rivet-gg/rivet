import {
	type HighlighterCore,
	createHighlighterCore,
	createOnigurumaEngine,
} from "shiki";
import { endWithSlash } from "../../lib/utils";
import {
	MessageSchema,
	ResponseSchema,
	type Response,
	type ReplErrorCode,
} from "./actor-worker-schema";

class ReplError extends Error {
	constructor(
		public readonly code: ReplErrorCode,
		message: string,
	) {
		super(message);
	}

	static unsupported() {
		return new ReplError("unsupported", "Actor unsupported");
	}
}

export let highlighter: HighlighterCore | undefined;

async function formatCode(code: string) {
	highlighter ??= await createHighlighterCore({
		themes: [import("shiki/themes/github-dark-default.mjs")],
		langs: [import("@shikijs/langs/typescript")],
		engine: createOnigurumaEngine(import("shiki/wasm")),
	});

	return highlighter.codeToTokens(code, {
		lang: "typescript",
		theme: "github-dark-default",
	});
}

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function evaluateCode(code: string, args: Record<string, unknown>) {
	const argsString = Object.keys(args);
	const argValues = Object.values(args);

	let jsCode: ReturnType<typeof toJs>;
	try {
		const program = fromJs(code, {
			module: true,
			allowAwaitOutsideFunction: true,
			allowReturnOutsideFunction: true,
		});

		const lastStatement = program.body[program.body.length - 1];
		if (lastStatement.type === "ExpressionStatement") {
			program.body[program.body.length - 1] = {
				type: "ReturnStatement",
				argument: lastStatement.expression,
			};
		}

		jsCode = toJs(program);
	} catch (e) {
		throw new ReplError("syntax", "Syntax error");
	}

	return new Function(
		"window",
		...argsString,
		`"use strict";
        return (async () => {
            ${jsCode.value}
    })()
    `,
	)({}, ...argValues);
}

const createConsole = (id: string) => {
	return new Proxy(
		{ ...console },
		{
			get(target, prop) {
				return (...args: unknown[]) => {
					respond({
						type: "log",
						id,
						data: {
							method: prop as "log" | "warn" | "error",
							data: args,
							timestamp: new Date().toISOString(),
						},
					});
					return Reflect.get(target, prop)(...args);
				};
			},
		},
	);
};

let init: null | { rpcs: string[] } = null;

addEventListener("message", async (event) => {
	const { success, error, data } = MessageSchema.safeParse(event.data);

	if (!success) {
		console.error("Malformed message", event.data, error);
		return;
	}

	if (data.type === "init") {
		init = {
			rpcs: [],
		};
	}

	if (data.type === "code") {
		const actor = init;
		if (!actor) {
			respond({
				type: "error",
				data: new Error("Actor not initialized"),
			});
			return;
		}

		try {
			const formatted = await formatCode(data.data);
			respond({
				type: "formatted",
				id: data.id,
				data: formatted,
			});

			const createRpc =
				(rpc: string) =>
				async (...args: unknown[]) => {
					const url = new URL(
						`rpc/${rpc}`,
						endWithSlash(actor.url.href),
					);
					const response = await fetch(url, {
						method: "POST",
						body: JSON.stringify({
							a: args,
						} satisfies Request),
					});

					if (!response.ok) {
						throw new Error("RPC failed");
					}

					const data = await response.json();
					return data.o;
				};

			const exposedActor = Object.fromEntries(
				init?.rpcs.map((rpc) => [rpc, createRpc(rpc)]) ?? [],
			);

			const evaluated = await evaluateCode(data.data, {
				console: createConsole(data.id),
				wait,
				actor: exposedActor,
			});
			return respond({
				type: "result",
				id: data.id,
				data: evaluated,
			});
		} catch (e) {
			return respond({
				type: "error",
				id: data.id,
				data: e,
			});
		}
	}
});

function respond(msg: Response) {
	return postMessage(ResponseSchema.parse(msg));
}
