"use client";
import { useActor } from "@rivet-gg/actor-client/unstable-react";
import { Suspense } from "react";
import type { default as ChatActor } from "../../actor/server-driven-ui";

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
