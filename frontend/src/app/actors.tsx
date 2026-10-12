import { useQuery } from "@tanstack/react-query";
import { useNavigate, useSearch } from "@tanstack/react-router";
import {
	type ActorId,
	ActorNotFound,
	ActorsActorDetails,
	ActorsActorEmptyDetails,
	ActorsListPreview,
	useDataProvider,
} from "@/components/actors";
import { ActorConfigTab } from "@/components/actors/actor-config-tab";

export function Actors({ actorId }: { actorId: string | undefined }) {
	return (
		<ActorsListPreview showDetails={!!actorId}>
			{actorId ? <Actor /> : <ActorsActorEmptyDetails />}
		</ActorsListPreview>
	);
}

function Actor() {
	const navigate = useNavigate();
	const { tab, actorId } = useSearch({ from: "/_context" });

	const { data, isError } = useQuery(
		useDataProvider().actorQueryOptions(actorId),
	);

	if (!data || isError) {
		return <ActorNotFound actorId={actorId as ActorId} />;
	}

	// TODO(agentOS-0.4): The inspector tabs do not work yet with agentOS 0.4, so show
	// only its metadata. Remove this branch once the tabs are fixed.
	if (data.name === "agentOS") {
		return <ActorConfigTab actorId={data.actorId} />;
	}

	return (
		<ActorsActorDetails
			actorId={data.actorId}
			tab={tab}
			onTabChange={(tab) =>
				navigate({
					to: ".",
					search: (old) => ({ ...old, tab }),
				})
			}
		/>
	);
}
