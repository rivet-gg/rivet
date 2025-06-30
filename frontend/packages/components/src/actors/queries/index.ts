import { infiniteQueryOptions, queryOptions } from "@tanstack/react-query";
import type {
	ActorLogEntry,
	Actor as InspectorActor,
} from "@rivetkit/core/inspector";
export { ActorFeature } from "@rivetkit/core/inspector";
export type { ActorLogEntry } from "@rivetkit/core/inspector";
import type { Rivet } from "@rivet-gg/api";
import z from "zod";

const ActorId = z.string().brand("ActorId");

export type Actor = Omit<InspectorActor, "id"> & {
	network?: Rivet.actor.Network;
} & { id: z.infer<typeof ActorId> };

export type ActorId = z.infer<typeof ActorId>;
export type ActorMetrics = {
	metrics: Record<string, number | null>;
	rawData: Record<string, number[]>;
	interval: number;
};

export const actorsQueryOptions = () =>
	infiniteQueryOptions<Actor[]>({
		queryKey: ["actors"],
		select: (data) => {
			return data.pages.flat();
		},
	});

export const actorsListQueryOptions = () =>
	infiniteQueryOptions({
		...actorsQueryOptions(),
		select: (data) => {
			return data.pages.flatMap((actors) =>
				actors.map((actor) => actor.id),
			);
		},
	});

export const actorsLisPaginationQueryOptions = () =>
	infiniteQueryOptions({
		...actorsQueryOptions(),
		notifyOnChangeProps: [
			"isFetchingNextPage",
			"fetchNextPage",
			"hasNextPage",
		],
	});

export const actorQueryOptions = (actorId: ActorId) =>
	queryOptions<Actor>({
		queryKey: ["actor", actorId],
	});

export const actorRegionQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorQueryOptions(actorId),
		select: (data) => data.region,
	});

export const actorTagsQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorQueryOptions(actorId),
		select: (data) => data.tags,
	});

export const actorCreatedAtQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorQueryOptions(actorId),
		select: (data) => (data.createdAt ? new Date(data.createdAt) : null),
	});

export const actorDestroyedAtQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorQueryOptions(actorId),
		select: (data) =>
			data.destroyedAt ? new Date(data.destroyedAt) : null,
	});

export const actorStatusQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorQueryOptions(actorId),
		select: (data) => getActorStatus(data),
	});

export const actorFeaturesQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorQueryOptions(actorId),
		select: (data) => data.features ?? [],
	});

export const actorLogsQueryOptions = (actorId: ActorId) =>
	queryOptions<ActorLogEntry[]>({
		queryKey: ["actor", actorId, "logs"],
	});

export const regionsQueryOptions = () =>
	queryOptions<Rivet.actor.Region[]>({
		queryKey: ["actor", "regions"],
	});

export const regionQueryOptions = (regionId: string | undefined) =>
	queryOptions({
		...regionsQueryOptions(),
		enabled: !!regionId,
		select: (regions) => regions.find((region) => region.id === regionId),
	});

export const actorNetworkQueryOptions = (actorId: ActorId) =>
	queryOptions<Rivet.actor.Network>({
		queryKey: ["actor", actorId, "network"],
	});

export const actorRuntimeQueryOptions = (actorId: ActorId) =>
	queryOptions<
		Pick<Rivet.actor.Actor, "runtime" | "lifecycle" | "resources" | "tags">
	>({
		queryKey: ["actor", actorId, "runtime"],
	});

export const actorNetworkPortsQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorNetworkQueryOptions(actorId),
		select: (data) => data.ports ?? [],
	});

export const actorBuildQueryOptions = (actorId: ActorId) =>
	queryOptions<Rivet.actor.Build>({
		queryKey: ["actor", actorId, "build"],
	});
export const actorMetricsQueryOptions = (actorId: ActorId) =>
	queryOptions<ActorMetrics>({
		queryKey: ["actor", actorId, "metrics"],
	});

export const actorDestroyMutationOptions = (actorId: ActorId) => ({
	mutationKey: ["actor", actorId, "destroy"],
});

export const actorGeneralQueryOptions = (actorId: ActorId) =>
	queryOptions({
		...actorQueryOptions(actorId),
		select: (data) => ({
			tags: data.tags,
			createdAt: data.createdAt ? new Date(data.createdAt) : null,
			destroyedAt: data.destroyedAt ? new Date(data.destroyedAt) : null,
			region: data.region,
		}),
	});

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
