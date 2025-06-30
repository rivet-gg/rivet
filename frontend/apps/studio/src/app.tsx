import {
	ConfigProvider,
	FullscreenLoading,
	ThirdPartyProviders,
	Toaster,
	TooltipProvider,
	getConfig,
} from "@rivet-gg/components";
import { PageLayout } from "@rivet-gg/components/layout";
import * as Sentry from "@sentry/react";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { Suspense } from "react";
import { routeTree } from "./routeTree.gen";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./queries/global";

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}

export const router = createRouter({
	basepath: import.meta.env.BASE_URL,
	routeTree,
	defaultStaleTime: Number.POSITIVE_INFINITY,
	defaultPendingComponent: PageLayout.Root.Skeleton,
	defaultPreloadStaleTime: 0,
	defaultOnCatch: (error) => {
		Sentry.captureException(error);
	},
});

function InnerApp() {
	return <RouterProvider router={router} />;
}

export function App() {
	return (
		<QueryClientProvider client={queryClient}>
			<ConfigProvider value={getConfig()}>
				<ThirdPartyProviders>
					<Suspense fallback={<FullscreenLoading />}>
						<TooltipProvider>
							<InnerApp />
						</TooltipProvider>
					</Suspense>
				</ThirdPartyProviders>

				<Toaster />
			</ConfigProvider>

			<ReactQueryDevtools client={queryClient} />
		</QueryClientProvider>
	);
}
