import { ActorsActorDetails } from "@rivet-gg/components/actors";
import { useExportActorLogsMutation } from "../../queries/actors/mutations";
import { useProject } from "../../data/project-context";
import { useEnvironment } from "../../data/environment-context";
import type { ActorAtom } from "@rivet-gg/components/actors";

interface ActorsActorDetailsWrapperProps {
	tab?: string;
	actor: ActorAtom;
	onTabChange?: (tab: string) => void;
}

export function ActorsActorDetailsWrapper({ 
	tab, 
	actor, 
	onTabChange 
}: ActorsActorDetailsWrapperProps) {
	const { nameId: projectNameId } = useProject();
	const { nameId: environmentNameId } = useEnvironment();
	const exportMutation = useExportActorLogsMutation();

	const handleExportLogs = async (actorId: string, typeFilter?: string, filter?: string) => {
		// Build query JSON for the API
		const query: any = {
			actorIds: [actorId],
		};

		// Add stream filter based on typeFilter
		if (typeFilter === "output") {
			query.stream = 0; // stdout
		} else if (typeFilter === "errors") {
			query.stream = 1; // stderr
		}

		// Add text search if filter is provided
		if (filter) {
			query.searchText = filter;
		}

		const result = await exportMutation.mutateAsync({
			projectNameId,
			environmentNameId,
			queryJson: JSON.stringify(query),
		});

		// Open the presigned URL in a new tab to download
		window.open(result.url, "_blank");
	};

	return (
		<ActorsActorDetails
			tab={tab}
			actor={actor}
			onTabChange={onTabChange}
			onExportLogs={handleExportLogs}
			isExportingLogs={exportMutation.isPending}
		/>
	);
}