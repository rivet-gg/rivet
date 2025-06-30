import { Badge } from "@rivet-gg/components";
import { useEffect, useRef } from "react";
import equal from "fast-deep-equal";

function usePreviousState<T>(state: T) {
	const ref = useRef<T>(state);
	useEffect(() => {
		ref.current = state;
	}, [state]);
	return ref.current;
}

interface ActorStateChangeIndicatorProps {
	state: unknown | undefined;
}

export function ActorStateChangeIndicator({
	state,
}: ActorStateChangeIndicatorProps) {
	const oldState = usePreviousState(state);
	const hasChanged = !equal(state, oldState);

	const ref = useRef<HTMLDivElement>(null);

	// biome-ignore lint/correctness/useExhaustiveDependencies: its okay, we only want to run this when state changes
	useEffect(() => {
		if (hasChanged && ref.current) {
			ref.current?.animate(
				[{ opacity: 1 }, { opacity: 1, offset: 0.7 }, { opacity: 0 }],
				{
					duration: 500,
					easing: "ease-in",
				},
			);
		}
	}, [state, hasChanged]);

	return (
		<Badge asChild>
			<div ref={ref} className="opacity-0">
				State changed
			</div>
		</Badge>
	);
}
