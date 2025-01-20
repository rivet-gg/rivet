import { assertNonNullable } from "@/lib/utils";
import { useSuspenseQuery } from "@tanstack/react-query";
import {
	type ReactNode,
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
	useState,
	useSyncExternalStore,
} from "react";
import { actorManagerUrlQueryOptions } from "../../../queries";
import ActorWorker from "./actor-repl.worker?worker";
import { ActorWorkerContainer } from "./actor-worker-container";

export const ActorWorkerContext = createContext<ActorWorkerContainer | null>(
	null,
);

export const useActorWorker = () => {
	const value = useContext(ActorWorkerContext);
	assertNonNullable(value);
	return value;
};

interface ActorWorkerContextProviderProps {
	actorId: string;
	projectNameId: string;
	environmentNameId: string;
	enabled?: boolean;
	children: ReactNode;
}

export const ActorWorkerContextProvider = ({
	children,
	actorId,
	enabled,
	projectNameId,
	environmentNameId,
}: ActorWorkerContextProviderProps) => {
	const { data: managerEndpoint } = useSuspenseQuery(
		actorManagerUrlQueryOptions({ projectNameId, environmentNameId }),
	);

	const [container] = useState<ActorWorkerContainer>(
		() => new ActorWorkerContainer(),
	);
	const workerRef = useRef<Worker>();

	// biome-ignore lint/correctness/useExhaustiveDependencies: we want to create worker on each of those props change
	useEffect(() => {
		if (!enabled) {
			return;
		}
		workerRef.current = new ActorWorker({ name: `actor-${actorId}` });
		container.setWorker(workerRef.current, {
			actorId,
			managerUrl: managerEndpoint,
		});

		return () => {
			workerRef.current?.terminate();
		};
	}, [actorId, managerEndpoint, enabled]);

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

export function useActorRpcs() {
	const container = useActorWorker();
	return useSyncExternalStore(
		useCallback(
			(cb) => {
				return container.subscribe(cb);
			},
			[container],
		),
		useCallback(() => {
			return container.getRpcs();
		}, [container]),
	);
}

export function useActorState() {
	const container = useActorWorker();
	return useSyncExternalStore(
		useCallback(
			(cb) => {
				return container.subscribe(cb);
			},
			[container],
		),
		useCallback(() => {
			return container.getState();
		}, [container]),
	);
}

export function useActorConnections() {
	const container = useActorWorker();
	return useSyncExternalStore(
		useCallback(
			(cb) => {
				return container.subscribe(cb);
			},
			[container],
		),
		useCallback(() => {
			return container.getConnections();
		}, [container]),
	);
}
