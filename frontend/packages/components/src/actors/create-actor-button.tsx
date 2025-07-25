import { Button, type ButtonProps, WithTooltip } from "@rivet-gg/components";
import { Icon, faPlus } from "@rivet-gg/icons";
import { useNavigate } from "@tanstack/react-router";
import { useActorsView } from "./actors-view-context-provider";
import { useQuery } from "@tanstack/react-query";
import { useManagerQueries } from "./manager-queries-context";

export function CreateActorButton(props: ButtonProps) {
	const navigate = useNavigate();

	const queries = useManagerQueries();
	const { data } = useQuery(useManagerQueries().buildsQueryOptions());

	const { copy, canCreate: contextAllowActorsCreation } = useActorsView();

	const canCreate =
		data &&
		data.length > 0 &&
		contextAllowActorsCreation &&
		queries.endpoint;

	if (!contextAllowActorsCreation) {
		return null;
	}

	const content = (
		<div>
			<Button
				disabled={!canCreate}
				size="sm"
				variant="ghost"
				onClick={() => {
					navigate({
						to: ".",
						search: (prev) => ({
							...prev,
							modal: "create-actor",
						}),
					});
				}}
				startIcon={<Icon icon={faPlus} />}
				{...props}
			>
				{copy.createActor}
			</Button>
		</div>
	);

	if (canCreate) {
		return content;
	}

	return (
		<WithTooltip
			trigger={content}
			content={
				(data && data.length <= 0) || !queries.endpoint
					? "Please deploy a build first."
					: copy.createActorUsingForm
			}
		/>
	);
}
