import type { Rivet } from "@rivet-gg/api";
import { infiniteQueryOptions, queryOptions } from "@tanstack/react-query";
import {
	type Actor,
	type ActorId,
	getActorStatus,
	type ActorLogEntry,
	type ActorMetrics,
} from "./queries";
import type { Builds, CreateActor } from "@rivetkit/core/inspector";
import { useContext, createContext } from "react";

const ACTORS_PER_PAGE = 10;

export const defaultManagerQueries = {
	queryClient: null as unknown as import("@tanstack/react-query").QueryClient,
	token: null as string | null,
	endpoint: "http://localhost:3000",
	async getManagerStatus(_: { url: string; token: string }): Promise<void> {
		throw new Error("Manager status query not implemented");
	},
	setToken(url: string, token: string) {
		this.token = token;
		sessionStorage.setItem(
			`rivetkit-token:${JSON.stringify({ url })}`,
			token,
		);
	},
	actorsQueryOptions() {
		return infiniteQueryOptions({
			queryKey: ["actors"],
			initialPageParam: undefined as ActorId | undefined,
			enabled: false,
			refetchInterval: 5000,
			queryFn: async () => {
				throw new Error("Actors query not implemented");
				// biome-ignore lint/correctness/noUnreachable: this way we tell tanstack query the query type
				return [] as Actor[];
			},
			getNextPageParam: (lastPage) => {
				if (
					!lastPage ||
					lastPage.length === 0 ||
					lastPage.length < ACTORS_PER_PAGE
				) {
					return undefined;
				}

				return lastPage[lastPage.length - 1].id;
			},
		});
	},

	actorsTagsQueryOptions() {
		return infiniteQueryOptions({
			...this.actorsQueryOptions(),
			select: (data) => {
				const tagsMap = new Map<string, Set<string>>();
				for (const actors of data.pages) {
					for (const actor of actors) {
						if (actor.tags) {
							for (const [key, value] of Object.entries(
								actor.tags,
							)) {
								if (!tagsMap.has(key)) {
									tagsMap.set(key, new Set());
								}
								// biome-ignore lint/style/noNonNullAssertion: we checked for that above
								tagsMap.get(key)!.add(value);
							}
						}
					}
				}
				const result: { key: string; value: string }[] = [];
				for (const [key, values] of tagsMap.entries()) {
					for (const value of values) {
						result.push({ key, value });
					}
				}
				return result;
			},
		});
	},

	buildsQueryOptions() {
		return queryOptions({
			queryKey: ["actors", "builds"],
			enabled: false,
			queryFn: async () => {
				throw new Error("Builds query not implemented");
				// biome-ignore lint/correctness/noUnreachable: this way we tell tanstack query the query type
				return [] as Builds;
			},
		});
	},

	actorsListQueryOptions() {
		return infiniteQueryOptions({
			...this.actorsQueryOptions(),
			refetchInterval: 5000,
			select: (data) => {
				return data.pages.flatMap((actors) =>
					actors.map((actor) => actor.id),
				);
			},
		});
	},

	actorsListPaginationQueryOptions() {
		return infiniteQueryOptions({
			...this.actorsQueryOptions(),
			select: (data) => {
				return data.pages.flatMap((actors) =>
					actors.map((actor) => actor.id),
				).length;
			},
		});
	},

	actorQueryOptions(actorId: ActorId) {
		return queryOptions<Actor>({
			queryFn: async () => {
				throw new Error("Actor query not implemented");
			},
			queryKey: ["actor", actorId],
			enabled: false,
		});
	},

	actorRegionQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data: Actor) => data.region,
		});
	},

	actorTagsQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data: Actor) => data.tags,
		});
	},

	actorCreatedAtQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data: Actor) =>
				data.createdAt ? new Date(data.createdAt) : null,
		});
	},

	actorDestroyedAtQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data: Actor) =>
				data.destroyedAt ? new Date(data.destroyedAt) : null,
		});
	},

	actorStatusQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data: Actor) => getActorStatus(data),
		});
	},

	actorFeaturesQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data: Actor) => data.features ?? [],
		});
	},
	actorGeneralQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data) => ({
				tags: data.tags,
				createdAt: data.createdAt ? new Date(data.createdAt) : null,
				destroyedAt: data.destroyedAt
					? new Date(data.destroyedAt)
					: null,
				region: data.region,
			}),
		});
	},
	actorBuildQueryOptions(actorId: ActorId) {
		return queryOptions<Rivet.builds.Build>({
			queryKey: ["actor", actorId, "build"],
			queryFn: async () => {
				throw new Error("Actor build query not implemented");
			},
			enabled: false,
		});
	},
	actorMetricsQueryOptions(
		actorId: ActorId,
		opts: { refetchInterval?: number } = {},
	) {
		return queryOptions<ActorMetrics>({
			queryKey: ["actor", actorId, "metrics"],
			queryFn: async () => {
				throw new Error("Actor metrics query not implemented");
			},
			enabled: false,
			...opts,
		});
	},
	actorDestroyMutationOptions(actorId: ActorId) {
		return {
			mutationKey: ["actor", actorId, "destroy"],
			mutationFn: async () => {
				throw new Error("Actor destroy mutation not implemented");
			},
		};
	},

	actorLogsQueryOptions(actorId: ActorId) {
		return queryOptions<ActorLogEntry[]>({
			queryKey: ["actor", actorId, "logs"],
		});
	},

	regionsQueryOptions() {
		return queryOptions<Rivet.regions.Region[]>({
			queryKey: ["actor", "regions"],
			queryFn: async () => {
				throw new Error("Regions query not implemented");
			},
		});
	},
	regionQueryOptions(regionId: string | undefined) {
		return queryOptions({
			...this.regionsQueryOptions(),
			enabled: !!regionId,
			select: (regions) =>
				regions.find((region) => region.id === regionId),
		});
	},
	actorNetworkQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data) => data.network,
		});
	},
	actorRuntimeQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: ({ runtime, lifecycle, resources, tags }) => ({
				runtime,
				lifecycle,
				resources,
				tags,
			}),
		});
	},
	actorNetworkPortsQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorNetworkQueryOptions(actorId),
			select: (data) => data.network?.ports ?? [],
		});
	},
	actorWorkerQueryOptions(actorId: ActorId) {
		return queryOptions({
			...this.actorQueryOptions(actorId),
			select: (data) => ({
				name: data.name,
				endpoint: this.endpoint,
				features: data.features ?? [],
				id: data.id,
				region: data.region,
				destroyedAt: data.destroyedAt
					? new Date(data.destroyedAt)
					: null,
				startedAt: data.startedAt ? new Date(data.startedAt) : null,
			}),
		});
	},
	managerStatusQueryOptions() {
		return queryOptions<boolean>({
			queryKey: ["managerStatus"],
			refetchInterval: 1000,
			enabled: false,
			retry: 0,
			queryFn: async () => {
				throw new Error("Manager status query not implemented");
			},
		});
	},
	createActorMutationOptions() {
		return {
			mutationKey: ["createActor"],
			mutationFn: async (_: CreateActor): Promise<void> => {
				throw new Error("Create actor mutation not implemented");
				return;
			},
		};
	},
};

const ManagerContext = createContext(defaultManagerQueries);

export const useManagerQueries = () => useContext(ManagerContext);

export const ManagerQueriesProvider = ManagerContext.Provider;

export const getManagerToken = (url: string) => {
	const token = sessionStorage.getItem(
		`rivetkit-token:${JSON.stringify({ url })}`,
	);
	return token;
};

export const setManagerToken = (url: string, token: string) => {
	sessionStorage.setItem(`rivetkit-token:${JSON.stringify({ url })}`, token);
};
