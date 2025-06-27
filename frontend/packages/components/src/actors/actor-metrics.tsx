import { useAtomValue } from "jotai";
import { selectAtom } from "jotai/utils";
import equal from "fast-deep-equal";
import { Dd, Dl, Dt, Flex } from "@rivet-gg/components";
import type { Actor, ActorAtom } from "./actor-context";

const selector = (a: Actor) => ({
	metrics: a.metrics,
});

export interface ActorMetricsProps {
	actor: ActorAtom;
}

export function ActorMetrics({ actor }: ActorMetricsProps) {
	const { metrics } = useAtomValue(selectAtom(actor, selector, equal));
	const metricsData = useAtomValue(metrics);

	const formatCpuUsage = (cpu: number | null) => {
		if (cpu === null) return "n/a";
		return `${(cpu * 100).toFixed(2)}%`;
	};

	const formatMemoryUsage = (memory: number | null) => {
		if (memory === null) return "n/a";
		return `${(memory / 1024 / 1024).toFixed(1)} MB`;
	};

	const isLoading = metricsData.status === "pending";
	const hasError = metricsData.status === "error";

	return (
		<div className="px-4 my-8">
			<h3 className="mb-2 font-semibold">Metrics</h3>
			<Flex gap="2" direction="col" className="text-xs">
				<Dl>
					<Dt>CPU Usage</Dt>
					<Dd className={hasError ? "text-destructive" : ""}>
						{isLoading ? "Loading..." : hasError ? "Error" : formatCpuUsage(metricsData.metrics.cpu)}
					</Dd>
					<Dt>Memory Usage</Dt>
					<Dd className={hasError ? "text-destructive" : ""}>
						{isLoading ? "Loading..." : hasError ? "Error" : formatMemoryUsage(metricsData.metrics.memory)}
					</Dd>
				</Dl>
			</Flex>
		</div>
	);
}