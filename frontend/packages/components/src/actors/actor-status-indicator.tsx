import { Ping, cn } from "@rivet-gg/components";
import type { ComponentPropsWithRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { type ActorStatus, actorStatusQueryOptions } from "./queries";

export const QueriedActorStatusIndicator = ({
	actorId,
	...props
}: {
	actorId: string;
} & ComponentPropsWithRef<"span">) => {
	const { data: status } = useQuery(actorStatusQueryOptions(actorId));
	return <ActorStatusIndicator status={status} {...props} />;
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
