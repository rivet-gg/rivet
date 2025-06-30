import { LiveBadge, ScrollArea } from "@rivet-gg/components";
import { ActorObjectInspector } from "./console/actor-inspector";
import { useQuery } from "@tanstack/react-query";
import { useActorConnectionsStream, type ActorId } from "./queries";
import { useManagerQueries } from "./manager-queries-context";
import { useActorQueries } from "./actor-queries-context";

interface ActorConnectionsTabProps {
	actorId: ActorId;
}

export function ActorConnectionsTab({ actorId }: ActorConnectionsTabProps) {
	const { data: destroyedAt } = useQuery(
		useManagerQueries().actorDestroyedAtQueryOptions(actorId),
	);

	const actorQueries = useActorQueries();
	const {
		data: { connections } = {},
		isError,
		isLoading,
	} = useQuery(actorQueries.actorConnectionsQueryOptions(actorId));

	useActorConnectionsStream(actorId);

	if (destroyedAt) {
		return (
			<div className="flex-1 flex items-center justify-center h-full text-center">
				Connections Preview is unavailable for inactive Actors.
			</div>
		);
	}

	if (isError) {
		return (
			<div className="flex-1 flex items-center justify-center h-full text-center">
				Connections Preview is currently unavailable.
				<br />
				See console/logs for more details.
			</div>
		);
	}

	if (isLoading) {
		return (
			<div className="flex-1 flex items-center justify-center h-full text-center">
				Loading connections...
			</div>
		);
	}

	return (
		<ScrollArea className="flex-1 w-full min-h-0 h-full">
			<div className="flex  justify-between items-center gap-1 border-b sticky top-0 p-2 bg-card z-[1] h-[45px]">
				<LiveBadge />
			</div>
			<div className="p-2">
				<ActorObjectInspector
					name="connections"
					data={connections}
					expandPaths={["$"]}
				/>
			</div>
		</ScrollArea>
	);
}
