import { ScrollArea } from "@rivet-gg/components";
import { ActorGeneral, type ActorGeneralProps } from "./actor-general";
import { ActorNetwork, type ActorNetworkProps } from "./actor-network";
import { ActorRuntime, type ActorRuntimeProps } from "./actor-runtime";

interface ActorConfigTabProps
	extends ActorRuntimeProps,
		ActorNetworkProps,
		ActorGeneralProps {}

export function ActorConfigTab(props: ActorConfigTabProps) {
	return (
		<ScrollArea className="overflow-auto h-full px-4">
			<ActorGeneral {...props} />
			<ActorNetwork {...props} />
			<ActorRuntime {...props} />
		</ScrollArea>
	);
}
