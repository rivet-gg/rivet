import {
  ActorId,
  createActorInspectorClient,
  Patch,
} from "@rivetkit/core/inspector";
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { compare } from "fast-json-patch";
import _ from "lodash";
import { useEffect } from "react";
import { fetchEventSource } from "@microsoft/fetch-event-source";

const createActorInspectorClientForActor = (actorId: ActorId | string) =>
  createActorInspectorClient("http://localhost:6420/registry/actors/inspect", {
    headers: {
      "X-RivetKit-Query": JSON.stringify({
        getForId: { actorId },
      }),
    },
  });

export const actorStateQueryOptions = (
  actorId: ActorId,
  { enabled }: { enabled: boolean } = { enabled: true }
) =>
  queryOptions({
    enabled,
    queryKey: ["actor", actorId, "state"],
    queryFn: async ({ queryKey: [, actorId] }) => {
      const client = createActorInspectorClientForActor(actorId);
      const response = await client.state.$get();

      if (!response.ok) {
        throw response;
      }
      return await response.json();
    },
  });

export const actorConnectionsQueryOptions = (
  actorId: ActorId,
  { enabled }: { enabled: boolean } = { enabled: true }
) =>
  queryOptions({
    enabled,
    refetchInterval: 2000,
    queryKey: ["actor", actorId, "connections"],
    queryFn: async ({ queryKey: [, actorId] }) => {
      const client = createActorInspectorClientForActor(actorId);
      const response = await client.connections.$get();

      if (!response.ok) {
        throw response;
      }
      return await response.json();
    },
  });

export const useActorStatePatchMutation = (
  actorId: ActorId,
  options?: Parameters<typeof useMutation>[1]
) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: any) => {
      const client = createActorInspectorClientForActor(actorId);

      const oldStateQuery = queryClient.getQueryData(
        actorStateQueryOptions(actorId).queryKey
      );

      const oldState = oldStateQuery?.state || {};

      const patches = compare(oldState, data);

      console.log({ oldState, data, patches });

      const response = await client.state.$patch({
        json: patches,
      });

      if (!response.ok) {
        throw response;
      }
      return await response.json();
    },
    onMutate: async (variables) => {
      queryClient.setQueryData(
        actorStateQueryOptions(actorId).queryKey,
        (data) => _.merge({}, data, { state: variables })
      );
    },
    ...options,
  });
};

export const useActorStateStream = (actorId: ActorId) => {
  const queryClient = useQueryClient();
  useEffect(() => {
    const controller = new AbortController();
    const client = createActorInspectorClientForActor(actorId);

    fetchEventSource(client.state.stream.$url().href, {
      signal: controller.signal,
      headers: {
        "X-RivetKit-Query": JSON.stringify({
          getForId: { actorId },
        }),
      },
      onmessage: (event) => {
        const data = JSON.parse(event.data);
        console.log("incoming", data);
        // queryClient.setQueryData(
        //   actorStateQueryOptions(actorId).queryKey,
        //   (data) => _.merge({}, data, { state: data })
        // );
      },
    }).catch((error) => console.error(error));

    return () => {
      controller.abort();
    };
  }, [actorId, queryClient]);
};
