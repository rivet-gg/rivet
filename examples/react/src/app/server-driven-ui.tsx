"use client";
import { useActor } from "./use-actor";
import { default as ChatActor } from "../../actor/server-driven-ui";
import { Suspense } from "react";

export function ServerDrivenUi() {
	const [{ actor }, { messages: Messages }] = useActor<ChatActor>({
		name: "server-driven-ui",
	});

	return (
		<>
			<Suspense>
				<Messages hello="hello" />
			</Suspense>
		</>
	);
}
