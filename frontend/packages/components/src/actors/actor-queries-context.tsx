import { queryOptions } from "@tanstack/react-query";
import type { ActorId } from "./queries";
import { useContext, createContext } from "react";
import { createActorInspectorClient } from "@rivetkit/core/inspector";

export const defaultActorQueries = {
	createActorInspectorHeaders: (actorId: ActorId | string) => ({
		"X-RivetKit-Query": JSON.stringify({
			getForId: { actorId },
		}),
	}),
	createActorInspector(actorId: ActorId | string) {
		return createActorInspectorClient(
			"http://localhost:6420/registry/actors/inspect",
			{
				headers: this.createActorInspectorHeaders(actorId),
			},
		);
	},
	actorPingQueryOptions(
		actorId: ActorId,
		opts: { enabled?: boolean; refetchInterval?: number | false } = {},
	) {
		return queryOptions({
			enabled: false,
			refetchInterval: 1000,
			...opts,
			queryKey: ["actor", actorId, "ping"],
			queryFn: async ({ queryKey: [, actorId] }) => {
				const client = this.createActorInspector(actorId);
				const response = await client.ping.$get();
				if (!response.ok) {
					throw response;
				}
				return await response.json();
			},
		});
	},

	actorStateQueryOptions(
		actorId: ActorId,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			enabled,
			queryKey: ["actor", actorId, "state"],
			queryFn: async ({ queryKey: [, actorId] }) => {
				const client = this.createActorInspector(actorId);
				const response = await client.state.$get();

				if (!response.ok) {
					throw response;
				}
				return (await response.json()) as {
					enabled: boolean;
					state: unknown;
				};
			},
		});
	},

	actorConnectionsQueryOptions(
		actorId: ActorId,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			enabled,
			queryKey: ["actor", actorId, "connections"],
			queryFn: async ({ queryKey: [, actorId] }) => {
				const client = this.createActorInspector(actorId);
				const response = await client.connections.$get();

				if (!response.ok) {
					throw response;
				}
				return await response.json();
			},
		});
	},

	actorDatabaseQueryOptions(
		actorId: ActorId,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			enabled,
			queryKey: ["actor", actorId, "database"],
			queryFn: async ({ queryKey: [, actorId] }) => {
				const client = this.createActorInspector(actorId);
				const response = await client.db.$get();

				if (!response.ok) {
					throw response;
				}
				return await response.json();
			},
		});
	},

	actorDatabaseEnabledQueryOptions(
		actorId: ActorId,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			...this.actorDatabaseQueryOptions(actorId, { enabled }),
			select: (data) => data.enabled,
			notifyOnChangeProps: ["data", "isError", "isLoading"],
		});
	},

	actorDatabaseTablesQueryOptions(
		actorId: ActorId,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			...this.actorDatabaseQueryOptions(actorId, { enabled }),
			select: (data) =>
				data.db?.map((table) => ({
					name: table.table.name,
					type: table.table.type,
					records: table.records,
				})) || [],
			notifyOnChangeProps: ["data", "isError", "isLoading"],
		});
	},

	actorDatabaseRowsQueryOptions(
		actorId: ActorId,
		table: string,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			enabled,
			staleTime: 0,
			gcTime: 5000,
			queryKey: ["actor", actorId, "database", table],
			queryFn: async ({ queryKey: [, actorId, , table] }) => {
				const client = this.createActorInspector(actorId);
				const response = await client.db.$post({
					json: { query: `SELECT * FROM ${table} LIMIT 500` },
				});
				if (!response.ok) {
					throw response;
				}
				return await response.json();
			},
		});
	},

	actorEventsQueryOptions(
		actorId: ActorId,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			enabled,
			queryKey: ["actor", actorId, "events"],
			queryFn: async ({ queryKey: [, actorId] }) => {
				const client = this.createActorInspector(actorId);
				const response = await client.events.$get();

				if (!response.ok) {
					throw response;
				}
				return await response.json();
			},
		});
	},

	actorRpcsQueryOptions(
		actorId: ActorId,
		{ enabled }: { enabled: boolean } = { enabled: true },
	) {
		return queryOptions({
			enabled,
			queryKey: ["actor", actorId, "rpcs"],
			queryFn: async ({ queryKey: [, actorId] }) => {
				const client = this.createActorInspector(actorId);
				const response = await client.rpcs.$get();

				if (!response.ok) {
					throw response;
				}
				return await response.json();
			},
		});
	},

	actorClearEventsMutationOptions(actorId: ActorId) {
		return {
			mutationKey: ["actor", actorId, "clear-events"],
			mutationFn: async () => {
				const client = this.createActorInspector(actorId);
				const response = await client.events.clear.$post();
				if (!response.ok) {
					throw response;
				}
				return await response.json();
			},
		};
	},
};

const ActorContext = createContext(defaultActorQueries);

export const useActorQueries = () => useContext(ActorContext);

export const ActorQueriesProvider = ActorContext.Provider;
