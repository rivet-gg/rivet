import { Button, ScrollArea } from "@rivet-gg/components";
import { Icon, faBooks } from "@rivet-gg/icons";
import { ActorMetrics } from "./actor-metrics";
import type { ActorId } from "./queries";

interface ActorMetricsTabProps {
	actorId: ActorId;
}

export function ActorMetricsTab(props: ActorMetricsTabProps) {
	return (
		<ScrollArea className="overflow-auto h-full">
			<div className="flex justify-end items-center gap-1 border-b sticky top-0 p-2 bg-card z-[1] h-[45px]">
				<Button
					variant="outline"
					size="sm"
					startIcon={<Icon icon={faBooks} />}
				>
					Documentation
				</Button>
			</div>
			<ActorMetrics {...props} />
		</ScrollArea>
	);
}