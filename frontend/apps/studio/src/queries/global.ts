import { toast } from "@rivet-gg/components";
import {
	MutationCache,
	QueryCache,
	QueryClient,
} from "@tanstack/react-query";

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
			gcTime: 1000 * 60 * 60 * 1,
			retry: 2,
			refetchOnWindowFocus: false,
			refetchOnReconnect: false,
		},
	},
	queryCache,
	mutationCache,
});
