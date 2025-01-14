import { ActorRepl } from "@/components/repl/repl";
import type { Rivet } from "@rivet-gg/api";
import { ScrollArea } from "@rivet-gg/components";
import { useSuspenseQuery } from "@tanstack/react-query";
import {
	actorManagerUrlQueryOptions,
	actorsRpcsQueryOptions,
} from "../../queries";

interface ActorRpcTabProps extends Rivet.actor.Actor {
	projectNameId: string;
	environmentNameId: string;
}

export function ActorRpcTab({
	network,
	projectNameId,
	environmentNameId,
	id: actorId,
}: ActorRpcTabProps) {
	const { data } = useSuspenseQuery(
		actorsRpcsQueryOptions({
			actorId,
			network,
			projectNameId,
			environmentNameId,
		}),
	);
	const { data: actorManagerUrl } = useSuspenseQuery(
		actorManagerUrlQueryOptions({ projectNameId, environmentNameId }),
	);

	return (
		<ScrollArea className="overflow-auto h-full px-4 my-2">
			<ActorRepl
				rpcs={data || []}
				actorId={actorId}
				managerUrl={actorManagerUrl}
			/>
		</ScrollArea>
	);
}
