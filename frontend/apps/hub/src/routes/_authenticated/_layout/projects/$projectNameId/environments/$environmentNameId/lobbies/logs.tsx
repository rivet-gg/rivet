import { ProjectMatchmakerListLobbyPreview } from "@/domains/project/components/matchmaker/matchmaker-list-lobby-preview";
import { useEnvironment } from "@/domains/project/data/environment-context";
import { useProject } from "@/domains/project/data/project-context";
import * as Layout from "@/domains/project/layouts/matchmaker-layout";
import { projectEnvironmentLogsLobbiesQueryOptions } from "@/domains/project/queries";
import { Card, CardContent, CardHeader, CardTitle } from "@rivet-gg/components";
import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { zodValidator } from "@tanstack/zod-adapter";
import { z } from "zod";

function MatchmakerLogsView() {
	const { namespaceId: environmentId } = useEnvironment();
	const { gameId: projectId } = useProject();

	const search = Route.useSearch();

	const { data: lobbies } = useSuspenseQuery(
		projectEnvironmentLogsLobbiesQueryOptions({ projectId, environmentId }),
	);

	return (
		<Card className="h-full max-h-full flex flex-col p-0">
			<CardHeader className="border-b">
				<CardTitle>Logs</CardTitle>
			</CardHeader>
			<CardContent className="flex-1 min-h-0 w-full p-0">
				<ProjectMatchmakerListLobbyPreview
					lobbies={lobbies}
					projectId={projectId}
					environmentId={environmentId}
					lobbyId={search.lobbyId}
				/>
			</CardContent>
		</Card>
	);
}

const searchSchema = z.object({
	lobbyId: z.string().optional(),
});

export const Route = createFileRoute(
	"/_authenticated/_layout/projects/$projectNameId/environments/$environmentNameId/lobbies/logs",
)({
	validateSearch: zodValidator(searchSchema),
	staticData: {
		layout: "full",
	},
	component: MatchmakerLogsView,
	pendingComponent: Layout.Content.Skeleton,
});
