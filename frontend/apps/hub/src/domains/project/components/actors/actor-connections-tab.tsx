import {
	Badge,
	Button,
	DocsSheet,
	LiveBadge,
	ScrollArea,
} from "@rivet-gg/components";
import { Icon, faInfoCircle } from "@rivet-gg/icons";
import { ActorObjectInspector } from "./console/actor-inspector";
import {
	useActorConnections,
	useActorWorkerStatus,
} from "./worker/actor-worker-context";

export function ActorConnectionsTab() {
	const status = useActorWorkerStatus();

	const connections = useActorConnections();

	console.log("UI: ActorConnectionsTab", connections);

	if (status.type === "error") {
		return (
			<div className="flex-1 flex items-center justify-center h-full text-xs text-center">
				Connections Preview is currently unavailable.
				<br />
				See console/logs for more details.
			</div>
		);
	}

	if (status.type !== "ready") {
		return (
			<div className="flex-1 flex items-center justify-center h-full text-xs text-center">
				Loading connections...
			</div>
		);
	}

	return (
		<ScrollArea className="flex-1 w-full min-h-0 h-full">
			<div className="p-2">
				<div className="flex justify-start items-center mb-2 gap-1">
					<DocsSheet title="State" path="docs/connections">
						<Button variant="ghost" size="icon-sm">
							<Icon icon={faInfoCircle} />
						</Button>
					</DocsSheet>
					<LiveBadge />
				</div>
				{connections.map((connection) => (
					<div
						key={connection.id}
						className="border rounded-md p-2 mb-2"
					>
						<Badge variant="outline">
							Connection{" "}
							<span className="font-mono-console ml-1">
								{connection.id}
							</span>
						</Badge>
						<div className="flex flex-col gap-2 mt-2 ml-1">
							<ActorObjectInspector
								name="state"
								data={connection.state}
								depth={2}
							/>
							<ActorObjectInspector
								name="subscriptions"
								data={connection.subscriptions}
								depth={2}
							/>
						</div>
					</div>
				))}
			</div>
		</ScrollArea>
	);
}
