import { FullscreenLoading } from "@rivet-gg/components";

import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { zodValidator } from "@tanstack/zod-adapter";
import { z } from "zod";

// function RootNotFoundComponent() {
// 	return (
// 		<PageLayout.Root>
// 			<Page title="Not found">
// 				<NotFoundComponent />
// 			</Page>
// 		</PageLayout.Root>
// 	);
// }

// function RootErrorComponent(props: ErrorComponentProps) {
// 	return (
// 		<PageLayout.Root>
// 			<Page title="Error!">
// 				<ErrorComponent {...props} />
// 			</Page>
// 		</PageLayout.Root>
// 	);
// }
function RootRoute() {
	return (
		<>
			<Outlet />
			{import.meta.env.DEV ? <TanStackRouterDevtools /> : null}
		</>
	);
}

const searchSchema = z.object({
	modal: z.enum(["go-to-actor", "feedback"]).or(z.string()).optional(),
	utm_source: z.string().optional(),
});

export const Route = createRootRouteWithContext()({
	validateSearch: zodValidator(searchSchema),
	component: RootRoute,
	pendingComponent: FullscreenLoading,
	wrapInSuspense: true,
});
