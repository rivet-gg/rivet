import { Rpc } from "@rivet-gg/actor";
import { RscActor } from "@rivet-gg/actor/unstable-react";
// @deno-types=npm:@types/react
import React from "react";

export default class ServerDrivenUi extends RscActor {
	messages(_rpc: Rpc<this>, props: Record<string, any>) {
		return (
			<div>
				<p>Hello from Rivet Actor!</p>
				<time>{new Date().toISOString()}</time>
				<br />
				<code>{JSON.stringify(props, null, 2)}</code>
			</div>
		);
	}
}
