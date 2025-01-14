import * as shiki from "shiki";

import { type ActorHandle, Client } from "@rivet-gg/actor-client";
import { fromJs } from "esast-util-from-js";
import { toJs } from "estree-util-to-js";
import githubDark from "shiki/themes/github-dark-default.mjs";
import { MessageSchema, ResponseSchema } from "./repl-schema";

export let highlighter: shiki.Highlighter | undefined;

async function formatCode(code: string) {
	highlighter ??= await shiki.getSingletonHighlighter({
		themes: [githubDark],
		langs: ["typescript"],
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

	const jsCode = toJs(program);

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
	const log = (level: string, message: unknown) => {
		postMessage(
			ResponseSchema.parse({
				type: "log",
				id,
				data: { level, message: JSON.stringify(message) },
			}),
		);
	};

	return {
		log: log.bind(null, "log"),
		info: log.bind(null, "info"),
		warn: log.bind(null, "warn"),
		error: log.bind(null, "error"),
	};
};

addEventListener("message", async (event) => {
	console.log(event);
	const { success, data } = MessageSchema.safeParse(event.data);
	console.log("msg", event.data);

	if (!success) {
		return;
	}

	if (data.type === "code") {
		let handle: ActorHandle | undefined;
		try {
			const formatted = await formatCode(data.data);
			postMessage(
				ResponseSchema.parse({
					type: "formatted",
					id: data.id,
					data: formatted,
				}),
			);

			const cl = new Client(data.managerUrl);
			handle = await Promise.race([
				cl.getWithId(data.actorId, {}),
				wait(5000).then(() => {
					throw new Error("Timeout");
				}),
			]);

			const actor = handle;

			if (!actor) {
				postMessage(
					ResponseSchema.parse({
						type: "error",
						id: data.id,
						data: new Error("Could not connect to actor"),
					}),
				);
				return;
			}

			const rpcs = Object.fromEntries(
				data.rpcs.map(
					(rpc) => [rpc, actor[rpc as keyof typeof actor]] as const,
				),
			);

			const evaluated = await evaluateCode(data.data, {
				console: createConsole(data.id),
				wait,
				actor: handle,
				...rpcs,
			});
			postMessage(
				ResponseSchema.parse({
					type: "result",
					id: data.id,
					data: JSON.stringify(evaluated),
				}),
			);
		} catch (e) {
			handle?.dispose();
			postMessage(
				ResponseSchema.parse({
					type: "error",
					id: data.id,
					data: e,
				}),
			);
		}
	}
});
