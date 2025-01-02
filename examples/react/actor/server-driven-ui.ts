import { Actor, Rpc } from "@rivet-gg/actor";
// @ts-ignore
import { renderToPipeableStream } from "react-server";
import { createElement } from "react";
import { PassThrough, Readable } from "node:stream";
import { getStreamAsArrayBuffer } from "get-stream";

interface State {
}

export default class ServerDrivenUi extends Actor<State> {
	#interval: number = 0;

	override _onInitialize() {
		return {};
	}

	protected override _onStart(): void | Promise<void> {
		this.#interval = setInterval(() => {
			this._broadcast("update");
		}, 1000);
	}

	protected override _shutdown(): Promise<void> {
		clearInterval(this.#interval);
		return super._shutdown();
	}

	render() {
		const rsc = renderToPipeableStream(
			createElement("div", { className: "actor-component" }, [
				createElement("p", {}, "Hello from Rivet Actor!"),
				createElement("p", {}, [
					"Rendered at ",
					createElement("time", {}, new Date().toISOString()),
				]),
				createElement("div", {}, [
					"Here are props passed to this component",
					createElement("code", {}, JSON.stringify({}, null, 2)),
				]),
			]),
		);

		return getStreamAsArrayBuffer(Readable.toWeb(
			rsc.pipe(new PassThrough()),
		) as ReadableStream<Uint8Array>);
	}
}
