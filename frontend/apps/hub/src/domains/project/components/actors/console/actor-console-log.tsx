import { memo } from "react";
import type { ReplCommand } from "../repl/actor-worker-container";
import { ActorConsoleLogFormatted } from "./actor-console-log-formatted";
import { ActorConsoleMessage } from "./actor-console-message";
import { ActorObjectInspector } from "./actor-inspector";

export const ActorConsoleLog = memo((props: ReplCommand) => {
	return (
		<>
			<ActorConsoleMessage
				timestamp={props.inputTimestamp}
				variant={
					props.status !== "success" && props.status !== "error"
						? "input-pending"
						: "input"
				}
			>
				{"formatted" in props && props.formatted ? (
					<ActorConsoleLogFormatted {...props.formatted} />
				) : null}
			</ActorConsoleMessage>
			{"error" in props ? (
				<ActorConsoleMessage
					variant="error"
					timestamp={props.outputTimestamp}
				>
					{props.error &&
					typeof props.error === "object" &&
					"toString" in props.error
						? props.error.toString()
						: JSON.stringify(props.error)}
				</ActorConsoleMessage>
			) : null}
			{props.logs?.map((log, index) => (
				<ActorConsoleMessage
					// biome-ignore lint/suspicious/noArrayIndexKey: static array
					key={index}
					variant={log.method}
					timestamp={log.timestamp}
				>
					{log.data?.map((element, key) => {
						if (typeof element === "string") {
							return (
								<span
									// biome-ignore lint/suspicious/noArrayIndexKey: static array
									key={key}
								>
									{element}
								</span>
							);
						}
						return (
							<ActorObjectInspector
								// biome-ignore lint/suspicious/noArrayIndexKey: static array
								key={index}
								data={element}
							/>
						);
					})}
				</ActorConsoleMessage>
			))}
			{"result" in props ? (
				<ActorConsoleMessage
					variant="output"
					timestamp={props.outputTimestamp}
				>
					{typeof props.result === "string" ? (
						props.result
					) : (
						<ActorObjectInspector data={props.result} />
					)}
				</ActorConsoleMessage>
			) : null}
		</>
	);
});
