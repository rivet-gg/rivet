import { useSuspenseQuery } from "@tanstack/react-query";
import { useProject } from "../../data/project-context";
import { projectMetadataQueryOptions } from "../../queries";
import { BillingSummary } from "./billing-summary";
import {
	H2,
	Card,
	CardHeader,
	CardTitle,
	CardContent,
} from "@rivet-gg/components";
import { BillingUsageProgress } from "./billing-usage-progress";
import { useBilling } from "./billing-context";
import { BillingPlans } from "./billing-plans";

interface ActorsBillingUsageProps {
	showPlans?: boolean;
	showUsage?: boolean;
}

export function ActorsBillingUsage({
	showPlans = true,
	showUsage = true,
}: ActorsBillingUsageProps) {
	const { gameId: projectId } = useProject();
	const {
		data: { legacyLobbiesEnabled },
	} = useSuspenseQuery(projectMetadataQueryOptions({ projectId }));

	const {
		credits: { max, used, overage },
		activePlan,
	} = useBilling();

	if (legacyLobbiesEnabled) {
		return <BillingPlans projectId={projectId} />;
	}

	return (
		<>
			<BillingSummary />
			{showPlans ? <BillingPlans projectId={projectId} /> : null}
			{showUsage ? (
				<>
					<H2 mt="8">Usage</H2>
					<Card>
						<CardHeader>
							<CardTitle>Actors</CardTitle>
						</CardHeader>
						<CardContent>
							<BillingUsageProgress
								max={max}
								used={used}
								overage={overage}
								plan={activePlan}
							/>
						</CardContent>
					</Card>
				</>
			) : null}
		</>
	);
}
