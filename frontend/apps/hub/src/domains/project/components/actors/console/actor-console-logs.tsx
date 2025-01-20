import { ScrollArea } from "@rivet-gg/components";
import { useActorDetailsSettings } from "../actor-details-settings";
import { useActorReplCommands } from "../worker/actor-worker-context";
import { ActorConsoleLog } from "./actor-console-log";

export function ActorConsoleLogs() {
	const commands = useActorReplCommands();

	const [settings] = useActorDetailsSettings();

	return (
		<ScrollArea className="w-full flex-1">
			{commands.map((log) => (
				<ActorConsoleLog
					{...log}
					key={log.key}
					inputTimestamp={
						settings.showTimestmaps ? log.inputTimestamp : undefined
					}
					outputTimestamp={
						settings.showTimestmaps
							? log.outputTimestamp
							: undefined
					}
				/>
			))}
		</ScrollArea>
	);
}
