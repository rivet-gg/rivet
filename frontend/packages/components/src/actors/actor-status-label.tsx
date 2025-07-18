import { useQuery } from "@tanstack/react-query";
import { type ActorStatus, actorStatusQueryOptions } from "./queries";

export const ACTOR_STATUS_LABEL_MAP = {
	unknown: "Unknown",
	starting: "Starting",
	running: "Running",
	stopped: "Stopped",
	crashed: "Crashed",
} satisfies Record<ActorStatus, string>;

export const ActorStatusLabel = ({ status }: { status: ActorStatus }) => {
	return <span>{ACTOR_STATUS_LABEL_MAP[status]}</span>;
};

export const QueriedActorStatusLabel = ({
	actorId,
}: {
	actorId: string;
}) => {
	const { data: status } = useQuery(actorStatusQueryOptions(actorId));
	return <ActorStatusLabel status={status} />;
};
