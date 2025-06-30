import {
	type ReactNode,
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
	useSyncExternalStore,
} from "react";
import { ActorWorkerContainer } from "./actor-worker-container";
import { assertNonNullable } from "../../lib/utils";
import { useQuery } from "@tanstack/react-query";
import { ActorFeature, type ActorId } from "../queries";
import { useManagerQueries } from "../manager-queries-context";
import { useActorQueries } from "../actor-queries-context";

export const ActorWorkerContext = createContext<ActorWorkerContainer | null>(
	null,
);

export const useActorWorker = () => {
	const value = useContext(ActorWorkerContext);
	assertNonNullable(value);
	return value;
};

interface ActorWorkerContextProviderProps {
	actorId: ActorId;
	children: ReactNode;
}
export const ActorWorkerContextProvider = ({
	children,
	actorId,
}: ActorWorkerContextProviderProps) => {
	const {
		data: { features, endpoint, name, destroyedAt, startedAt } = {},
	} = useQuery(useManagerQueries().actorWorkerQueryOptions(actorId));
	const enabled =
		(features?.includes(ActorFeature.Console) &&
			!destroyedAt &&
			!!startedAt) ??
		false;

	const actorQueries = useActorQueries();
	const {
		data: { rpcs } = {},
	} = useQuery(actorQueries.actorRpcsQueryOptions(actorId, { enabled }));

	const [container] = useState<ActorWorkerContainer>(
		() => new ActorWorkerContainer(),
	);

	// biome-ignore lint/correctness/useExhaustiveDependencies: we want to create worker on each of those props change
	useEffect(() => {
		const ctrl = new AbortController();

		if (enabled) {
			container.init({
				actorId,
				endpoint,
				name,
				signal: ctrl.signal,
				rpcs,
			});
		}

		return () => {
			ctrl.abort();
			container.terminate();
		};
	}, [actorId, enabled, rpcs, endpoint, name]);

	return (
		<ActorWorkerContext.Provider value={container}>
			{children}
		</ActorWorkerContext.Provider>
	);
};

export function useActorReplCommands() {
	const container = useActorWorker();
	return useSyncExternalStore(
		useCallback(
			(cb) => {
				return container.subscribe(cb);
			},
			[container],
		),
		useCallback(() => {
			return container.getCommands();
		}, [container]),
	);
}

export function useActorWorkerStatus() {
	const container = useActorWorker();
	return useSyncExternalStore(
		useCallback(
			(cb) => {
				return container.subscribe(cb);
			},
			[container],
		),
		useCallback(() => {
			return container.getStatus();
		}, [container]),
	);
}
