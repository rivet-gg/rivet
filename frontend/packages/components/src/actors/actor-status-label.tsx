import { useQuery } from "@tanstack/react-query";
import type { ActorId, ActorStatus } from "./queries";
import { useManagerQueries } from "./manager-queries-context";

export const ACTOR_STATUS_LABEL_MAP = {
	unknown: "Unknown",
	starting: "Starting",
	running: "Running",
	stopped: "Stopped",
	crashed: "Crashed",
} satisfies Record<ActorStatus, string>;

export const ActorStatusLabel = ({ status }: { status?: ActorStatus }) => {
	return <span>{status ? ACTOR_STATUS_LABEL_MAP[status] : "Unknown"}</span>;
};

export const QueriedActorStatusLabel = ({ actorId }: { actorId: ActorId }) => {
	const { data: status, isError } = useQuery(
		useManagerQueries().actorStatusQueryOptions(actorId),
	);
	return <ActorStatusLabel status={isError ? "unknown" : status} />;
};
