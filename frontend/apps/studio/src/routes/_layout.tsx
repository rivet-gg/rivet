import { createFileRoute, Outlet } from "@tanstack/react-router";
import * as Layout from "@/components/layout";
import z from "zod";
import { zodValidator } from "@tanstack/zod-adapter";

export const Route = createFileRoute("/_layout")({
	component: RouteComponent,
	validateSearch: zodValidator(
		z.object({
			u: z.string().optional(),
		}),
	),
});

function RouteComponent() {
	return (
		<Layout.Root>
			<Layout.VisibleInFull>
				<Layout.Header />
				<Layout.Main>
					<div className="size-full bg-card">
						<Outlet />
					</div>
				</Layout.Main>
			</Layout.VisibleInFull>
			<Layout.Footer />
		</Layout.Root>
	);
}
