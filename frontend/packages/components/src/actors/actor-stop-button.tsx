import { Button, WithTooltip } from "@rivet-gg/components";
import { Icon, faXmark } from "@rivet-gg/icons";

import { useMutation, useQuery } from "@tanstack/react-query";
import { type ActorId } from "./queries";
import { useManagerQueries } from "./manager-queries-context";

interface ActorStopButtonProps {
	actorId: ActorId;
}

export function ActorStopButton({ actorId }: ActorStopButtonProps) {
	const { data: destroyedAt } = useQuery(
		useManagerQueries().actorDestroyedAtQueryOptions(actorId),
	);

	const { mutate, isPending } = useMutation(
		useManagerQueries().actorDestroyMutationOptions(actorId),
	);

	if (destroyedAt) {
		return null;
	}

	return (
		<WithTooltip
			trigger={
				<Button
					isLoading={isPending}
					variant="destructive"
					size="icon-sm"
					onClick={() => mutate()}
				>
					<Icon icon={faXmark} />
				</Button>
			}
			content="Stop Actor"
		/>
	);
}
