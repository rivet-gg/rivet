import {
	ActorFeature,
	ActorsActorDetails,
	ActorsActorEmptyDetails,
	ActorsListPreview,
} from "@rivet-gg/components/actors";
import { useNavigate, useSearch } from "@tanstack/react-router";

export function Actors({ actorId }: { actorId: string | undefined }) {
	return (
		<ActorsListPreview>
			{actorId ? (
				<Actor />
			) : (
				<ActorsActorEmptyDetails
					features={[
						ActorFeature.Config,
						ActorFeature.State,
						ActorFeature.Connections,
					]}
				/>
			)}
		</ActorsListPreview>
	);
}

function Actor() {
	const navigate = useNavigate();
	const { tab, actorId } = useSearch({ from: "/_layout/" });

	if (!actorId) {
		return null;
	}
	return (
		<ActorsActorDetails
			actorId={actorId}
			tab={tab}
			onTabChange={(tab) => {
				navigate({
					to: ".",
					search: (old) => ({ ...old, tab }),
				});
			}}
		/>
	);
}
