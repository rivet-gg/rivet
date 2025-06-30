import { Button, WithTooltip } from "@rivet-gg/components";
import { Icon, faXmark } from "@rivet-gg/icons";

import { useMutation, useQuery } from "@tanstack/react-query";
import {
	actorDestroyedAtQueryOptions,
	actorDestroyMutationOptions,
	type ActorId,
} from "./queries";

interface ActorStopButtonProps {
	actorId: ActorId;
}

export function ActorStopButton({ actorId }: ActorStopButtonProps) {
	const { data: destroyedAt } = useQuery(
		actorDestroyedAtQueryOptions(actorId),
	);

	const { mutate, isPending } = useMutation(
		actorDestroyMutationOptions(actorId),
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
