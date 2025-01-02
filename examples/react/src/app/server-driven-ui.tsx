"use client";
import { useActor } from "./use-actor";
import type { default as ChatActor } from "../../actor/server-driven-ui";
import { ActorHandle as RivetActorHandle } from "@rivet-gg/actor-client";
import { ReactNode } from "react";

export type ActorHandle = RivetActorHandle<ChatActor>;

export class ChatStore {
	#handle: ActorHandle;
	#output: ReactNode;

	#listeners: (() => void)[] = [];

	constructor(handle: ActorHandle) {
		this.#handle = handle;

		this.#handle.render().then((output) => {
			this.#output = output;
			this.#notify?.();
		});
	}

	update = async () => {
		this.#output = await this.#handle.render();
	};

	subscribe = (cb: () => void) => {
		this.#listeners.push(cb);
		const unsub = this.#handle.on("update", (output: ReactNode) => {
			this.#output = output;
			this.#notify?.();
		});

		return () => {
			unsub();
			this.#listeners = this.#listeners.filter((l) => l !== cb);
		};
	};

	#notify = () => {
		this.#listeners?.forEach((cb) => cb());
	};

	render = () => {
		return this.#output;
	};
}

export function ServerDrivenUi() {
	const state = useActor<ChatActor>({ name: "server-driven-ui" });

	if (!("actor" in state) || state.isLoading) {
		if ("error" in state) {
			return <div>Error while loading actor, see console for more details</div>;
		}
		return <div>Loading...</div>;
	}

	return (
		<>
			<Content actor={state.actor} />
		</>
	);
}

function Content({ actor }: { actor: ActorHandle }) {
	const store = new ChatStore(actor);

	return <>{store.render()}</>;
}
