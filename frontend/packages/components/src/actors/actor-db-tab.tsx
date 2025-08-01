import type { ActorId } from "./queries";
import { useQuery } from "@tanstack/react-query";
import { Info } from "./actor-state-tab";
import { DocsSheet } from "../docs-sheet";
import { Button } from "../ui/button";
import { ActorDatabase } from "./actor-database";
import { useManagerQueries } from "./manager-queries-context";
import { useActorQueries } from "./actor-queries-context";

interface ActorDatabaseTabProps {
	actorId: ActorId;
}

export function ActorDatabaseTab({ actorId }: ActorDatabaseTabProps) {
	const { data: destroyedAt } = useQuery(
		useManagerQueries().actorDestroyedAtQueryOptions(actorId),
	);

	const actorQueries = useActorQueries();
	const {
		data: isEnabled,
		isLoading,
		isError,
	} = useQuery(actorQueries.actorDatabaseEnabledQueryOptions(actorId));

	if (destroyedAt) {
		return <Info>Database Studio is unavailable for inactive Actors.</Info>;
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

	if (!isEnabled) {
		return (
			<Info>
				<p>
					Database Studio is not enabled for this Actor. <br /> You
					can enable it by providing a valid database connection
					provider.
				</p>
			</Info>
		);
	}

	return (
		<div className="flex-1 w-full min-h-0 min-w-0 h-full flex flex-col">
			<ActorDatabase actorId={actorId} />
		</div>
	);
}
