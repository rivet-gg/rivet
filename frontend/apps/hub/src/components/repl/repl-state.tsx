import { useCallback, useState, useSyncExternalStore } from "react";
import {
	type FormattedCode,
	type Log,
	type Message,
	ResponseSchema,
} from "./repl-schema";
import ReplWorker from "./repl.worker?worker";

export type ReplCommand = {
	logs: Log[];
	code: string;
	key: string;
} & (
	| { status: "pending" }
	| { status: "formatted"; formatted: FormattedCode }
	| { status: "success"; formatted: FormattedCode; result: unknown }
	| { status: "error"; formatted: FormattedCode | undefined; error: unknown }
);

class WorkerState {
	commands: ReplCommand[];

	#listeners: (() => void)[] = [];
	#worker: Worker;

	constructor() {
		this.#worker = new ReplWorker();
		this.commands = [];

		this.#worker.addEventListener(
			"message",
			this.#handleMessage.bind(this),
		);
		this.#worker.addEventListener("error", (error) => {
			console.log(error, error.message, error.error);
		});
	}

	runCode({
		code,
		actorId,
		managerUrl,
		rpcs,
	}: Omit<Message, "id" | "data" | "type"> & { code: string }) {
		const key = Date.now().toString();
		this.commands = [
			...this.commands,
			{ status: "pending", code, key, logs: [] },
		];

		this.#worker.postMessage({
			type: "code",
			data: code,
			actorId,
			managerUrl,
			rpcs,
			id: key,
		} satisfies Message);
		this.#update();
	}

	getSnapshot() {
		return this.commands;
	}

	subscribe(cb: () => void) {
		this.#listeners.push(cb);
		return () => {
			this.#listeners = this.#listeners.filter(
				(listener) => listener !== cb,
			);
		};
	}

	#handleMessage(event: MessageEvent) {
		console.log(event.data);
		const { success, data } = ResponseSchema.safeParse(event.data);

		if (!success) {
			return;
		}

		if (data.type === "formatted") {
			const command = this.commands.find(
				(command) => command.key === data.id,
			);
			if (command) {
				const newCommand = {
					...command,
					status: "formatted",
					formatted: data.data,
				};
				Object.assign(command, newCommand);
				this.commands = [...this.commands];
				this.#update();
			}
		}

		if (data.type === "result") {
			const command = this.commands.find(
				(command) => command.key === data.id,
			);
			if (command) {
				const newCommand = {
					...command,
					status: "success",
					result: data.data,
				};
				Object.assign(command, newCommand);
				this.commands = [...this.commands];
				this.#update();
			}
		}

		if (data.type === "log") {
			const command = this.commands.find(
				(command) => command.key === data.id,
			);
			if (command) {
				const newCommand = {
					...command,
					logs: [...command.logs, data.data],
				};
				Object.assign(command, newCommand);
				this.commands = [...this.commands];
				this.#update();
			}
		}

		if (data.type === "error") {
			const command = this.commands.find(
				(command) => command.key === data.id,
			);
			if (command) {
				const newCommand = {
					...command,
					status: "error",
					error: data.data,
				};
				Object.assign(command, newCommand);
				this.commands = [...this.commands];
				this.#update();
			}
		}
	}

	#update() {
		for (const listener of this.#listeners) {
			listener();
		}
	}
}

export function useRepl() {
	const [state] = useState(() => new WorkerState());
	return [
		useSyncExternalStore(
			useCallback((fn) => state.subscribe(fn), [state.subscribe]),
			useCallback(() => state.getSnapshot(), [state.getSnapshot]),
		),
		useCallback(
			(...args) => state.runCode(...args),
			[state.runCode],
		) as WorkerState["runCode"],
	] as const;
}
