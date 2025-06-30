import type { ActorId } from "./queries";
import { useQuery } from "@tanstack/react-query";
import { ActorEvents } from "./actor-events";
import { useManagerQueries } from "./manager-queries-context";
import { Info } from "./actor-state-tab";
import { useActorQueries } from "./actor-queries-context";

export type EventsTypeFilter = "action" | "subscription" | "broadcast" | "send";

interface ActorEventsTabProps {
	actorId: ActorId;
}

export function ActorEventsTab({ actorId }: ActorEventsTabProps) {
	const { data: destroyedAt } = useQuery(
		useManagerQueries().actorDestroyedAtQueryOptions(actorId),
	);

	const { isError, isLoading } = useQuery(
		useActorQueries().actorEventsQueryOptions(actorId),
	);

	if (destroyedAt) {
		return (
			<div className="flex-1 flex flex-col gap-2 items-center justify-center h-full text-center col-span-full py-8">
				State Preview is unavailable for inactive Actors.
			</div>
		);
	}

	if (isError) {
		return (
			<Info>
				Database Studio is currently unavailable.
				<br />
				See console/logs for more details.
			</Info>
		);
	}

	if (isLoading) {
		return <Info>Loading...</Info>;
	}

	return <ActorEvents actorId={actorId} />;
}
