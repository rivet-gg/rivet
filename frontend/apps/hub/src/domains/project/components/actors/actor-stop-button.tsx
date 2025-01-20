import { Button, WithTooltip } from "@rivet-gg/components";
import { Icon, faHexagonXmark } from "@rivet-gg/icons";
import { useSuspenseQuery } from "@tanstack/react-query";
import {
	actorDestroyedAtQueryOptions,
	useDestroyActorMutation,
} from "../../queries";

interface ActorStopButtonProps {
	projectNameId: string;
	environmentNameId: string;
	actorId: string;
}

export function ActorStopButton({
	projectNameId,
	environmentNameId,
	actorId,
}: ActorStopButtonProps) {
	const { data: destroyedAt } = useSuspenseQuery(
		actorDestroyedAtQueryOptions({
			projectNameId,
			environmentNameId,
			actorId,
		}),
	);
	const { mutate, isPending: isDestroying } = useDestroyActorMutation();

	if (destroyedAt) {
		return null;
	}
	return (
		<WithTooltip
			trigger={
				<Button
					isLoading={isDestroying}
					variant="ghost"
					size="icon-sm"
					onClick={() =>
						mutate({
							projectNameId,
							environmentNameId,
							actorId,
						})
					}
				>
					<Icon icon={faHexagonXmark} className="!size-4" />
				</Button>
			}
			content="Stop Actor"
		/>
	);
}
