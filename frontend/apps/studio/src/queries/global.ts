import { toast } from "@rivet-gg/components";
import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";
import {
	actorsQueryOptions,
	actorQueryOptions,
	type Actor,
	ActorFeature,
	actorLogsQueryOptions,
	regionsQueryOptions,
} from "@rivet-gg/components/actors";

const queryCache = new QueryCache();

const mutationCache = new MutationCache({
	onError(error, variables, context, mutation) {
		if (mutation.meta?.hideErrorToast) {
			return;
		}
		toast.error("Error occurred while performing the operation.", {
			description: error.message,
		});
	},
});

export const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			staleTime: 5 * 1000,
			gcTime: 1000 * 60 * 60 * 24,
			retry: 2,
			refetchOnWindowFocus: false,
			refetchOnReconnect: false,
		},
	},
	queryCache,
	mutationCache,
});

const actors = [
	{
		id: "actor-1",
		name: "Actor 1",
		key: ["string", "string"],
		region: "local",
		createdAt: new Date().toISOString(),
		features: [ActorFeature.Logs, ActorFeature.Config],
	},
	{
		id: "actor-2",
		name: "Actor 2",
		key: ["string", "string"],
		region: "remote",
		createdAt: new Date().toISOString(),
	},
	{
		id: "actor-3",
		name: "Actor 3",
		key: ["string", "string"],
		region: "remote",
		createdAt: new Date().toISOString(),
	},
];

queryClient.setQueryDefaults(actorsQueryOptions().queryKey, {
	queryFn: () => {
		return actors;
	},
	// @ts-expect-error
	getNextPageParam: (lastPage) => {
		return undefined;
	},
});

queryClient.setQueryDefaults(actorQueryOptions("actor-1").queryKey, {
	queryFn: () => {
		return actors[0];
	},
});

queryClient.setQueryDefaults(actorLogsQueryOptions("actor-1").queryKey, {
	queryFn: () => {
		return [
			{
				id: "log-1",
				message: "Log message 1",
				level: "info",
				timestamp: new Date().toISOString(),
			},
			{
				id: "log-2",
				message: "Log message 2",
				level: "error",
				timestamp: new Date().toISOString(),
			},
		];
	},
});

queryClient.setQueryDefaults(regionsQueryOptions().queryKey, {
	queryFn: () => {
		return [
			{
				id: "local",
				name: "Local",
			},
		];
	},
});
queryClient.setQueryData(regionsQueryOptions().queryKey, () => {
	return [
		{
			id: "local",
			name: "Local",
		},
	];
});
