import { createFileRoute, Outlet } from "@tanstack/react-router";
import * as Layout from "@/components/layout";
import z from "zod";
import { zodValidator } from "@tanstack/zod-adapter";
import {
	type ActorId,
	ActorQueriesProvider,
	createActorInspectorClient,
	createManagerInspectorClient,
	defaultActorQueries,
	defaultManagerQueries,
	getManagerToken,
	ManagerQueriesProvider,
	setManagerToken,
	useDialog,
} from "@rivet-gg/components/actors";
import { Suspense, useEffect, useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { usePostHog } from "posthog-js/react";
import { FEEDBACK_FORM_ID } from "@rivet-gg/components";

export const Route = createFileRoute("/_layout")({
	component: RouteComponent,
	validateSearch: zodValidator(
		z.object({
			u: z.string().optional(),
			t: z.string().optional(),
		}),
	),
});

function ensureTrailingSlash(url: string): string {
	if (url.endsWith("/")) {
		return url;
	}
	return `${url}/`;
}

function RouteComponent() {
	const { u: url } = Route.useSearch();

	const [token, setToken] = useState(() => getManagerToken(url || ""));

	const queryClient = useQueryClient();
	useEffect(() => {
		queryClient.invalidateQueries();
	}, [token, url]);

	const managerQueries = useMemo(() => {
		const provideToken = (newUrl: string, newToken: string) => {
			setToken(newToken);
			setManagerToken(newUrl, newToken);
		};

		const createClient = (url: string, token: string) => {
			const newUrl = new URL(url);
			if (!newUrl.pathname.endsWith("registry/inspect")) {
				if (!newUrl.pathname.endsWith("registry")) {
					newUrl.pathname = `${ensureTrailingSlash(newUrl.pathname)}registry`;
				}
				if (!newUrl.pathname.endsWith("inspect")) {
					newUrl.pathname = `${ensureTrailingSlash(newUrl.pathname)}inspect`;
				}
			}

			return createManagerInspectorClient(newUrl.href, {
				headers: { Authorization: `Bearer ${token}` },
			});
		};

		const getManagerStatus = async ({
			url,
			token,
		}: {
			url: string;
			token: string;
		}) => {
			const client = createClient(url, token);
			const res = await client.ping.$get();
			if (!res.ok) {
				throw new Error("Failed to fetch manager status");
			}
		};

		if (url) {
			if (!token) {
				return {
					...defaultManagerQueries,
					queryClient,
					endpoint: url,
					token,
					setToken: provideToken,
					getManagerStatus,
				};
			}
			const client = createClient(url, token);
			return {
				...defaultManagerQueries,
				endpoint: url,
				queryClient,
				token,
				getManagerStatus,
				setToken: provideToken,
				managerStatusQueryOptions() {
					return {
						...defaultManagerQueries.managerStatusQueryOptions(),
						enabled: true,
						queryFn: async ({ signal }) => {
							const status = await client.ping.$get({ signal });
							if (!status.ok) {
								throw new Error(
									"Failed to fetch manager status",
								);
							}
							return true;
						},
					};
				},
				regionsQueryOptions() {
					return {
						...defaultManagerQueries.regionsQueryOptions(),
						enabled: true,
						queryFn: async () => {
							return [{ id: "local", name: "Local" }];
						},
					};
				},
				actorQueryOptions(actorId: ActorId) {
					return {
						...defaultManagerQueries.actorQueryOptions(actorId),
						enabled: true,
						queryFn: async ({ signal }) => {
							const actor = await client.actor[":id"].$get({
								param: { id: actorId },
								// @ts-expect-error
								signal,
							});
							if (!actor.ok) {
								throw new Error(
									`Failed to fetch actor with ID: ${actorId}`,
								);
							}
							return await actor.json();
						},
					};
				},
				actorsQueryOptions() {
					return {
						...defaultManagerQueries.actorsQueryOptions(),
						enabled: true,
						queryFn: async ({ signal, pageParam }) => {
							const actors = await client.actors.$get({
								query: { cursor: pageParam, limit: 10 },
								signal,
							});
							if (!actors.ok) {
								throw new Error("Failed to fetch actors");
							}
							return await actors.json();
						},
					};
				},
				buildsQueryOptions() {
					return {
						...defaultManagerQueries.buildsQueryOptions(),
						enabled: true,
						queryFn: async ({ signal }) => {
							const builds = await client.builds.$get({ signal });
							if (!builds.ok) {
								throw new Error("Failed to fetch builds");
							}
							return await builds.json();
						},
					};
				},
				createActorMutationOptions() {
					return {
						...defaultManagerQueries.createActorMutationOptions(),
						mutationFn: async (data) => {
							const response = await client.actors.$post({
								json: data,
							});
							if (!response.ok) {
								throw new Error("Failed to create actor");
							}
						},
						onSuccess: () => {
							queryClient.invalidateQueries({
								queryKey: this.actorsQueryOptions().queryKey,
							});
						},
					};
				},
			};
		}
		return {
			...defaultManagerQueries,
			queryClient,
			token,
			setToken: provideToken,
			getManagerStatus,
		};
	}, [url, token]);

	const actorQueries = useMemo(() => {
		if (!url || !token) {
			return defaultActorQueries;
		}
		const newUrl = new URL(url);
		if (!newUrl.pathname.endsWith("registry/inspect")) {
			if (!newUrl.pathname.endsWith("registry")) {
				newUrl.pathname = `${ensureTrailingSlash(newUrl.pathname)}registry`;
			}
			if (!newUrl.pathname.endsWith("inspect")) {
				newUrl.pathname = `${ensureTrailingSlash(newUrl.pathname)}inspect`;
			}
		}
		newUrl.pathname = newUrl.pathname.replace(
			"/registry/inspect",
			"/registry/actors/inspect",
		);
		return {
			...defaultActorQueries,
			createActorInspectorHeaders(actorId: ActorId | string) {
				return {
					"X-RivetKit-Query": JSON.stringify({
						getForId: { actorId },
					}),
					Authorization: `Bearer ${token}`,
				};
			},
			createActorInspector(actorId: ActorId | string) {
				return createActorInspectorClient(newUrl.href, {
					headers: this.createActorInspectorHeaders(actorId),
				});
			},
		} satisfies typeof defaultActorQueries;
	}, [url, token]);

	return (
		<ManagerQueriesProvider value={managerQueries}>
			<ActorQueriesProvider value={actorQueries}>
				<Suspense>
					<Modals />
				</Suspense>
				<Layout.Root>
					<Layout.VisibleInFull>
						<Layout.Header />
						<Layout.Main>
							<div className="size-full bg-card">
								<Outlet />
							</div>
						</Layout.Main>
					</Layout.VisibleInFull>
					<Layout.Footer />
				</Layout.Root>
			</ActorQueriesProvider>
		</ManagerQueriesProvider>
	);
}

function Modals() {
	const search = Route.useSearch();
	const navigate = Route.useNavigate();

	const posthog = usePostHog();

	const FeedbackDialog = useDialog.Feedback.Dialog;
	const GoToActorDialog = useDialog.GoToActor.Dialog;
	const CreateActorDialog = useDialog.CreateActor.Dialog;

	const { modal, utm_source } = search;

	const handleOnOpenChange = (value: boolean) => {
		if (!value) {
			navigate({ search: (old) => ({ ...old, modal: undefined }) });
		} else {
			posthog.capture("survey shown", { $survey_id: FEEDBACK_FORM_ID });
		}
	};

	return (
		<>
			<FeedbackDialog
				source={utm_source}
				dialogProps={{
					open: modal === "feedback",
					onOpenChange: handleOnOpenChange,
				}}
			/>
			<GoToActorDialog
				onSubmit={(actorId) => {
					navigate({
						to: ".",
						search: (old) => ({
							...old,
							actorId,
							modal: undefined,
						}),
					});
				}}
				dialogProps={{
					open: modal === "go-to-actor",
					onOpenChange: (value) => {
						if (!value) {
							navigate({
								search: (old) => ({
									...old,
									modal: undefined,
								}),
							});
						}
					},
				}}
			/>
			<CreateActorDialog
				dialogProps={{
					open: modal === "create-actor",
					onOpenChange: (value) => {
						if (!value) {
							navigate({
								search: (old) => ({
									...old,
									modal: undefined,
								}),
							});
						}
					},
				}}
			/>
		</>
	);
}
