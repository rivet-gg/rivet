import { useAtomValue } from "jotai";
import { selectAtom } from "jotai/utils";
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
import { toast } from "sonner";
import { useQuery } from "@tanstack/react-query";
import {
	ActorFeature,
	type ActorId,
	actorWorkerQueryOptions,
} from "../queries";

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
		data: { endpoint, features, destroyedAt, startedAt } = {},
	} = useQuery(actorWorkerQueryOptions(actorId));

	const enabled =
		(features?.includes(ActorFeature.Console) &&
			!destroyedAt &&
			startedAt) ??
		false;

	const [container] = useState<ActorWorkerContainer>(
		() => new ActorWorkerContainer(),
	);

	// biome-ignore lint/correctness/useExhaustiveDependencies: we want to create worker on each of those props change
	useEffect(() => {
		const ctrl = new AbortController();

		if (enabled && endpoint) {
			container.init({
				actorId,
				signal: ctrl.signal,
			});
		} else {
			toast.dismiss("ac-ws-reconnect");
		}

		return () => {
			ctrl.abort();
			container.terminate();
		};
	}, [actorId, endpoint, enabled]);

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
