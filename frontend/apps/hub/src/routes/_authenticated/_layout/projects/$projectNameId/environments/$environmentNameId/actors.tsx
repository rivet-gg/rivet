import { GetStarted } from "@/components/get-started";
import { ActorsListPreview } from "@/domains/project/components/actors/actors-list-preview";
import * as Layout from "@/domains/project/layouts/servers-layout";
import { actorsCountQueryOptions } from "@/domains/project/queries";
import { Icon, faActors } from "@rivet-gg/icons";
import { useSuspenseInfiniteQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { zodValidator } from "@tanstack/zod-adapter";
import { z } from "zod";

function ProjectActorsRoute() {
	const {
		environment: { nameId: environmentNameId },
		project: { nameId: projectNameId },
	} = Route.useRouteContext();
	const { data } = useSuspenseInfiniteQuery(
		actorsCountQueryOptions({ projectNameId, environmentNameId }),
	);
	const { actorId } = Route.useSearch();

	if (data === 0) {
		return (
			<div className="w-full h-full flex flex-col justify-center">
				<div className="flex flex-col justify-center my-8">
					<Icon icon={faActors} className="text-6xl mx-auto my-4" />
					<h3 className="text-center font-bold text-xl max-w-md mb-2 mx-auto">
						Deploy your first Actor
					</h3>
					<p className="text-center text-muted-foreground max-w-sm mx-auto">
						Install Rivet to get started or use an existing template
						to get started.
					</p>
				</div>
				<GetStarted />
			</div>
		);
	}

	return (
		<div className="flex flex-col h-[calc(100vh-6.5rem)] bg-card -mx-4 -my-4">
			<ActorsListPreview
				projectNameId={projectNameId}
				environmentNameId={environmentNameId}
				actorId={actorId}
			/>
		</div>
	);
}

const searchSchema = z.object({
	actorId: z.string().optional(),
	tab: z.string().optional(),
});

export const Route = createFileRoute(
	"/_authenticated/_layout/projects/$projectNameId/environments/$environmentNameId/actors",
)({
	validateSearch: zodValidator(searchSchema),
	staticData: {
		layout: "full",
	},
	component: ProjectActorsRoute,
	pendingComponent: Layout.Content.Skeleton,
});
