import { Button, ScrollArea } from "@rivet-gg/components";
import { useRef } from "react";
import { useActorWorker } from "../worker/actor-worker-context";
import { ActorConsoleMessage } from "./actor-console-message";
import { ReplInput, type ReplInputRef, replaceCode } from "./repl-input";
import type { ActorId } from "../queries";
import { useQuery } from "@tanstack/react-query";
import { useActorQueries } from "../actor-queries-context";

interface ActorConsoleInputProps {
	actorId: ActorId;
}

export function ActorConsoleInput({ actorId }: ActorConsoleInputProps) {
	const worker = useActorWorker();

	const actorQueries = useActorQueries();
	const {
		data: { rpcs = [] } = {},
	} = useQuery(actorQueries.actorRpcsQueryOptions(actorId));

	const ref = useRef<ReplInputRef>(null);

	return (
		<div className="border-t w-full max-h-20 flex flex-col">
			<ScrollArea className="w-full flex-1">
				<ActorConsoleMessage variant="input" className="border-b-0">
					<ReplInput
						ref={ref}
						className="w-full"
						rpcs={rpcs}
						onRun={(code) => {
							worker.run(code);
						}}
					/>
				</ActorConsoleMessage>
				<div className="flex items-center mt-1 pb-1">
					<div className="flex flex-wrap gap-1">
						{rpcs.map((rpc) => (
							<Button
								variant="outline"
								size="xs"
								key={rpc}
								onClick={() => {
									if (!ref.current?.view) {
										return;
									}
									replaceCode(
										ref.current.view,
										`actor.${rpc}(`,
									);
								}}
								className="rounded-full"
								startIcon={
									<span className="bg-secondary px-1 rounded-full">
										RPC
									</span>
								}
							>
								<span className="font-mono-console">
									actor.{rpc}(...)
								</span>
							</Button>
						))}
					</div>
				</div>
			</ScrollArea>
		</div>
	);
}
