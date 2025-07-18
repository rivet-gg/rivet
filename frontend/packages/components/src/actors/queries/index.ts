import { infiniteQueryOptions, queryOptions } from "@tanstack/react-query";
import type {
  ActorLogEntry,
  Actor as InspectorActor,
} from "@rivetkit/core/inspector";
export { ActorFeature } from "@rivetkit/core/inspector";
export type { ActorLogEntry } from "@rivetkit/core/inspector";
import type { Rivet } from "@rivet-gg/api";
import { actorsDataProvider } from "../actor-context";
import { ActorId } from "@rivetkit/core/inspector";

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

const ACTORS_PER_PAGE = 10;

export const actorsQueryOptions = () =>
  infiniteQueryOptions<Actor[]>({
    queryKey: ["actors"],
    initialPageParam: undefined as ActorId | undefined,
    queryFn: async ({ signal, pageParam }) => {
      const actors = await actorsDataProvider.get().getActors(
        { cursor: pageParam as string, limit: ACTORS_PER_PAGE },
        {
          signal,
        }
      );

      if (!actors) {
        throw new Error("Failed to fetch actors");
      }

      return actors;
    },
    getNextPageParam: (lastPage) => {
      if (lastPage.length === 0 || lastPage.length < ACTORS_PER_PAGE) {
        return undefined;
      }

      return lastPage[lastPage.length - 1].id;
    },
  });

export const actorsListQueryOptions = () =>
  infiniteQueryOptions({
    ...actorsQueryOptions(),
    refetchInterval: 5000,
    select: (data) => {
      return data.pages.flatMap((actors) => actors.map((actor) => actor.id));
    },
  });

export const actorsLisPaginationQueryOptions = () =>
  infiniteQueryOptions({
    ...actorsQueryOptions(),
    notifyOnChangeProps: ["isFetchingNextPage", "fetchNextPage", "hasNextPage"],
  });

export const actorQueryOptions = (actorId: ActorId) =>
  queryOptions<Actor>({
    queryKey: ["actor", actorId],
    queryFn: async ({ signal }) => {
      const actor = await actorsDataProvider.get().getActor(actorId, {
        signal,
      });

      if (!actor) {
        throw new Error(`Actor with ID ${actorId} not found`);
      }

      return actor;
    },
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
    select: (data) => (data.destroyedAt ? new Date(data.destroyedAt) : null),
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
  queryOptions<Rivet.regions.Region[]>({
    queryKey: ["actor", "regions"],
    queryFn: async ({ signal }) => {
      return await actorsDataProvider.get().getRegions({
        signal,
      });
    },
  });

export const regionQueryOptions = (regionId: string | undefined) =>
  queryOptions({
    ...regionsQueryOptions(),
    enabled: !!regionId,
    select: (regions) => regions.find((region) => region.id === regionId),
  });

export const actorNetworkQueryOptions = (actorId: ActorId) =>
  queryOptions({
    ...actorQueryOptions(actorId),
    select: (data) => data.network,
  });

export const actorRuntimeQueryOptions = (actorId: ActorId) =>
  queryOptions({
    ...actorQueryOptions(actorId),
    select: ({ runtime, lifecycle, resources, tags }) => ({
      runtime,
      lifecycle,
      resources,
      tags,
    }),
  });

export const actorNetworkPortsQueryOptions = (actorId: ActorId) =>
  queryOptions({
    ...actorNetworkQueryOptions(actorId),
    select: (data) => data.network?.ports ?? [],
  });

export const actorWorkerQueryOptions = (actorId: ActorId) =>
  queryOptions({
    ...actorQueryOptions(actorId),
    select: (data) => ({
      features: data.features ?? [],
      id: data.id,
      region: data.region,
      destroyedAt: data.destroyedAt ? new Date(data.destroyedAt) : null,
      startedAt: data.startedAt ? new Date(data.startedAt) : null,
    }),
  });

export const actorBuildQueryOptions = (actorId: ActorId) =>
  queryOptions<Rivet.builds.Build>({
    queryKey: ["actor", actorId, "build"],
  });
export const actorMetricsQueryOptions = (
  actorId: ActorId,
  opts: { refetchInterval?: number } = {}
) =>
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
  actor: Pick<Actor, "createdAt" | "startedAt" | "destroyedAt">
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
