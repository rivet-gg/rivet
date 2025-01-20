import { Flex, Text } from "@rivet-gg/components";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { ActorsActorDetails } from "./actors-actor-details";

interface ActorsActorDetailsPanelProps {
	projectNameId: string;
	environmentNameId: string;
	actorId: string | undefined;
}

export function ActorsActorDetailsPanel({
	projectNameId,
	environmentNameId,
	actorId,
}: ActorsActorDetailsPanelProps) {
	const currentTab = useSearch({
		from: "/_authenticated/_layout/projects/$projectNameId/environments/$environmentNameId/actors",
		select: (state) => state.tab,
	});

	const navigate = useNavigate();

	if (!actorId) {
		return (
			<Flex items="center" justify="center" className="h-full">
				<Text textAlign="center">
					Please select an actor from the list.
				</Text>
			</Flex>
		);
	}

	return (
		<ActorsActorDetails
			tab={currentTab}
			projectNameId={projectNameId}
			environmentNameId={environmentNameId}
			actorId={actorId}
			onTabChange={(tab) => {
				navigate({
					from: "/projects/$projectNameId/environments/$environmentNameId/actors",
					to: ".",
					search: (old) => ({ ...old, tab }),
				});
			}}
		/>
	);
}
ActorsActorDetailsPanel.Skeleton = ActorsActorDetails.Skeleton;
