"use client";
import { useActor } from "@rivet-gg/actor-client/unstable-react";
import { default as ChatActor } from "../../actor/server-driven-ui";
import { Suspense } from "react";

export function ServerDrivenUi() {
	const [, { messages: Messages }] = useActor<ChatActor>({
		name: "server-driven-ui",
	});

	return (
		<>
			<Suspense>
				<Messages limit={10} />
			</Suspense>
		</>
	);
}
