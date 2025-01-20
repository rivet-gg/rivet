import { ActorsList } from "./actors-list";

interface ActorsListPanelProps {
	projectNameId: string;
	environmentNameId: string;
	actorId: string | undefined;
}

export function ActorsListPanel({
	actorId,
	projectNameId,
	environmentNameId,
}: ActorsListPanelProps) {
	return (
		<>
			<ActorsList
				projectNameId={projectNameId}
				environmentNameId={environmentNameId}
				actorId={actorId}
			/>
		</>
	);
}
