import type { Actor as InspectorActor } from "@rivetkit/core/inspector";
export { ActorFeature } from "@rivetkit/core/inspector";
export type { ActorLogEntry } from "@rivetkit/core/inspector";
import type { Rivet } from "@rivet-gg/api";
import type { ActorId } from "@rivetkit/core/inspector";

export type { ActorId };

export type Actor = Omit<InspectorActor, "id"> & {
	network?: Rivet.actors.Network;
	runtime?: Rivet.actors.Runtime;
	lifecycle?: Rivet.actors.Lifecycle;
	resources?: Rivet.actors.Resources;
	tags?: Record<string, string>;
} & { id: ActorId };

export type ActorMetrics = {
	metrics: Record<string, number | null>;
	rawData: Record<string, number[]>;
	interval: number;
};

export * from "./actor";

export type ActorStatus =
	| "starting"
	| "running"
	| "stopped"
	| "crashed"
	| "unknown";

export function getActorStatus(
	actor: Pick<Actor, "createdAt" | "startedAt" | "destroyedAt">,
): ActorStatus {
	const { createdAt, startedAt, destroyedAt } = actor;

	if (createdAt && !startedAt && !destroyedAt) {
		return "starting";
	}

	if (createdAt && startedAt && !destroyedAt) {
		return "running";
	}

	if (createdAt && startedAt && destroyedAt) {
		return "stopped";
	}

	if (createdAt && !startedAt && destroyedAt) {
		return "crashed";
	}

	return "unknown";
}
