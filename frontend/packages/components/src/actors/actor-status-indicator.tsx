import { Ping, cn } from "@rivet-gg/components";
import type { ComponentPropsWithRef } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ActorId, ActorStatus } from "./queries";
import { useManagerQueries } from "./manager-queries-context";

export const QueriedActorStatusIndicator = ({
	actorId,
	...props
}: {
	actorId: ActorId;
} & ComponentPropsWithRef<"span">) => {
	const { data: status, isError } = useQuery(
		useManagerQueries().actorStatusQueryOptions(actorId),
	);

	return (
		<ActorStatusIndicator
			status={isError ? "stopped" : status}
			{...props}
		/>
	);
};

interface ActorStatusIndicatorProps extends ComponentPropsWithRef<"span"> {
	status: ActorStatus | undefined;
}

export const ActorStatusIndicator = ({
	status,
	...props
}: ActorStatusIndicatorProps) => {
	if (status === "running") {
		return (
			<Ping
				variant="success"
				{...props}
				className={cn("relative right-auto", props.className)}
			/>
		);
	}

	return (
		<span
			{...props}
			className={cn(
				"size-2 rounded-full",
				{
					"bg-blue-600 animate-pulse": status === "starting",
					"bg-destructive": status === "crashed",
					"bg-foreground/10": status === "stopped",
				},
				props.className,
			)}
		/>
	);
};
