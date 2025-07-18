import { toast } from "@rivet-gg/components";
import {
	MutationCache,
	QueryCache,
	QueryClient,
	queryOptions,
} from "@tanstack/react-query";
import {
	createManagerInspectorClient,
	actorsDataProvider,
} from "@rivet-gg/components/actors";

const inspectorManager = createManagerInspectorClient(
	"http://localhost:6420/registry/inspect",
);

actorsDataProvider.set({
	getActors: async ({ cursor }, { signal }) => {
		const response = await inspectorManager.actors.$get({
			query: {
				cursor,
				limit: 10,
			},
			signal,
		});

		if (!response.ok) {
			throw new Error("Failed to fetch actors");
		}

		return await response.json();
	},
	getActor: async (id, { signal }) => {
		const response = await inspectorManager.actor[":id"].$get({
			param: { id },
		});

		if (!response.ok) {
			throw new Error("Failed to fetch actor");
		}

		return await response.json();
	},
	getActorLogs: async (id, { signal }) => {
		throw new Error("Not implemented");
	},
	getRegions: async () => {
		return [{ id: "local", name: "Local" }];
	},
});

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
			gcTime: 60 * 1000,
			retry: 3,
			refetchOnWindowFocus: true,
			refetchOnReconnect: false,
		},
	},
	queryCache,
	mutationCache,
});

export const managerStatusQueryOptions = () =>
	queryOptions({
		queryKey: ["managerStatus"],
		refetchInterval: 1000,
		retry: 0,
		queryFn: async () => {
			return inspectorManager.ping.$get();
		},
	});
