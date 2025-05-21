import { groupBillingQueryOptions } from "@/domains/group/queries";
import { projectQueryOptions } from "@/domains/project/queries";
import { useSuspenseQuery } from "@tanstack/react-query";
import { Billing } from "../components/billing/billing";
import { MissingPaymentMethod } from "../components/billing/billing-missing-payment-method";
import { ActorsBillingUsage } from "../components/billing/actors-billing-usage";

interface BillingViewProps {
	projectId: string;
}

export function BillingView({ projectId }: BillingViewProps) {
	const {
		data: { developerGroupId },
	} = useSuspenseQuery(projectQueryOptions(projectId));

	const { data: groupBilling } = useSuspenseQuery(
		groupBillingQueryOptions(developerGroupId),
	);

	if (!groupBilling.group.paymentMethodAttachedTs) {
		return (
			<>
				<MissingPaymentMethod
					projectId={projectId}
					groupId={developerGroupId}
				/>
				<ActorsBillingUsage showPlans={false} showUsage={false} />
			</>
		);
	}

	return <Billing projectId={projectId} groupId={developerGroupId} />;
}
