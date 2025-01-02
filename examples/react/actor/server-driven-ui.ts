import { Actor, Rpc } from "@rivet-gg/actor";
import * as rsc from "react-server";
import getStream from "get-stream";
import { createElement } from "react";
import { inOutStream } from "./test.ts";

interface State {}

export default class ServerDrivenUi extends Actor<State> {
	#interval: number = 0;

	override _onInitialize() {
		return {};
	}

	protected override _onStart(): void | Promise<void> {
		this.#interval = setInterval(() => {
			this._broadcast("__rsc");
		}, 1000);
	}

	protected override _shutdown(): Promise<void> {
		clearInterval(this.#interval);
		return super._shutdown();
	}

	messages(_rpc: Rpc<this>, props: Record<string, any>) {
		const stream = rsc.renderToPipeableStream(
			createElement(
				"div",
				{},
				createElement(
					"p",
					{},
					"Hello from Rivet Actorfffffff!",
					createElement("time", {}, new Date().toISOString()),
					createElement("br"),
					createElement(
						"code",
						{},
						JSON.stringify(props, null, 2),
					),
				),
			),
		);

		const { readable, ...writable } = inOutStream();

		stream.pipe(writable);

		return getStream(readable);
	}
}
