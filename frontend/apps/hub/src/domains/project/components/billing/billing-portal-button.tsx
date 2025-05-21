import { Button, type ButtonProps } from "@rivet-gg/components";
import { billingPortalUrlQueryOptions } from "../../queries";
import { useSuspenseQuery } from "@tanstack/react-query";

interface BillingPortalButtonProps extends ButtonProps {
	groupId: string;
	intent: "general" | "payment_method_update";
}

export function BillingPortalButton({
	groupId,
	intent,
	children,
	...props
}: BillingPortalButtonProps) {
	const { data } = useSuspenseQuery(
		billingPortalUrlQueryOptions({ groupId, intent }),
	);
	return (
		<Button type="button" {...props} asChild>
			<a href={data.stripeSessionUrl} target="_blank" rel="noreferrer">
				{children}
			</a>
		</Button>
	);
}
