import { Button, Kbd, ScrollArea, WithTooltip } from "@rivet-gg/components";
import { Icon, faInfo } from "@rivet-gg/icons";
import { useRef } from "react";
import { useActorRpcs, useActorWorker } from "../worker/actor-worker-context";
import { ActorConsoleMessage } from "./actor-console-message";
import { ReplInput, type ReplInputRef, replaceCode } from "./repl-input";

export function ActorConsoleInput() {
	const worker = useActorWorker();
	const rpcs = useActorRpcs();
	const ref = useRef<ReplInputRef>(null);

	return (
		<div className="border-t w-full max-h-20 flex">
			<ScrollArea className="w-full flex-1">
				<ActorConsoleMessage variant="input">
					<ReplInput
						ref={ref}
						className="w-full"
						rpcs={rpcs}
						onRun={(code) => {
							worker.run(code);
						}}
					/>
					<div className="flex flex-wrap mt-1 gap-1">
						<WithTooltip
							trigger={
								<Button
									variant="outline"
									size="icon-xs"
									className="rounded-full"
								>
									<Icon icon={faInfo} />
								</Button>
							}
							content={
								<div className="text-xs">
									<Kbd>Shift+Enter</Kbd> to execute code on
									Actor.
								</div>
							}
						/>
						{rpcs.map((rpc) => (
							<Button
								variant="outline"
								size="xs"
								key={rpc}
								onClick={() => {
									if (!ref.current?.view) {
										return;
									}
									replaceCode(ref.current.view, `${rpc}(`);
								}}
								className="rounded-full"
								startIcon={
									<span className="bg-secondary px-1 rounded-full">
										RPC
									</span>
								}
							>
								{rpc}(...)
							</Button>
						))}
					</div>
				</ActorConsoleMessage>
			</ScrollArea>
		</div>
	);
}
