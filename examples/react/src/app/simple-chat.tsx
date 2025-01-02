"use client";
import type { ActorHandle } from "@rivet-gg/actor-client";
import { useEffect, useState } from "react";
import { default as ChatActor } from "../../actor/simple-chat";
import { useActor } from "./use-actor";

export function SimpleChat() {
	const [{ isLoading, error, actor }] = useActor<ChatActor>({ name: "simple-chat" });

	if (isLoading) {
		return <>Loading...</>;
	}

	if (error) {
		return <>{JSON.stringify(error)}</>;
	}

	if (!actor) {
		return <>No actor.</>;
	}

	return (
		<>
			<ChatContent actor={actor} />
		</>
	);
}

function ChatContent({ actor }: { actor: ActorHandle<ChatActor> }) {
	const sendMessage = (message: string) => {
		actor.sendMessage(message);
	};

	return (
		<>
			<ChatMessageForm onSubmit={sendMessage} />
			<ChatMessages actor={actor} />
		</>
	);
}

function ChatMessages({ actor }: { actor: ActorHandle<ChatActor> }) {
	const [messages, setMessages] = useState<string[]>(() => []);

	useEffect(() => {
		actor.getMessages().then((messages) => {
			setMessages(messages);
		});
		return actor.on("newMessage", (response: { messages: string[] }) => {
			setMessages(response.messages);
		});
	}, [actor]);

	return (
		<div>
			{messages.map((msg, i) => (
				<div key={i}>{msg}</div>
			))}
		</div>
	);
}

export function ChatMessageForm({ onSubmit }: { onSubmit: (message: string) => void }) {
	return (
		<form
			onSubmit={(e) => {
				e.preventDefault();
				const data = new FormData(e.currentTarget);
				onSubmit(data.get("message") as string);
				e.currentTarget.reset();
			}}
		>
			<input type="text" name="message" />
			<button type="submit">Send</button>
		</form>
	);
}
