import { Button, WithTooltip } from "@rivet-gg/components";
import { Icon, faXmark } from "@rivet-gg/icons";

import type { Actor, ActorAtom, DestroyActorAtom } from "./actor-context";
import { useAtomValue } from "jotai";
import { selectAtom } from "jotai/utils";
import equal from "fast-deep-equal";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
	actorDestroyedAtQueryOptions,
	actorDestroyMutationOptions,
} from "./queries";

const selector = (a: Actor) => ({
	destroyedAt: a.destroyedAt,
	destroy: a.destroy,
});

interface ActorStopButtonProps {
	actorId: string;
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
					onClick={mutate}
				>
					<Icon icon={faXmark} />
				</Button>
			}
			content="Stop Actor"
		/>
	);
}
