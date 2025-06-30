import { Actors } from "@/components/actors";
import {
	Button,
	Card,
	CardContent,
	CardHeader,
	CardTitle,
	DocsSheet,
	H1,
} from "@rivet-gg/components";
import { Icon, faNodeJs, faReact } from "@rivet-gg/icons";
import { createFileRoute } from "@tanstack/react-router";
import { zodValidator } from "@tanstack/zod-adapter";
import { useEffect, useLayoutEffect, useRef } from "react";
import { z } from "zod";
import { useManagerQueries } from "@rivet-gg/components/actors";
import { useQuery } from "@tanstack/react-query";
import { ConnectionForm } from "@/components/connection-form";
import { docsLinks } from "@/content/data";

export const Route = createFileRoute("/_layout/")({
	component: RouteComponent,
	validateSearch: zodValidator(
		z
			.object({
				actorId: z.string().optional(),
				tab: z.string().optional(),
				url: z.string().optional(),
			})
			.and(z.record(z.string(), z.any())),
	),
});

function RouteComponent() {
	const { actorId, u = "http://localhost:8080", t } = Route.useSearch();

	const navigate = Route.useNavigate();
	const { setToken, token, ...queries } = useManagerQueries();

	const { isSuccess } = useQuery(queries.managerStatusQueryOptions());
	const previouslyConnected = useRef(isSuccess);

	const ref = useRef<HTMLFormElement>(null);

	useLayoutEffect(() => {
		if (u && t) {
			ref.current?.requestSubmit();
		}
	}, [u, t]);

	useEffect(() => {
		if (isSuccess && !previouslyConnected.current) {
			previouslyConnected.current = true;
		}
	}, [isSuccess]);

	if ((token && previouslyConnected.current) || isSuccess) {
		return <Actors actorId={actorId} />;
	}

	return (
		<div className="w-full h-full flex flex-col items-center justify-center">
			<H1 className="mb-8">Rivet Studio</H1>
			<Card className="max-w-md w-full mb-6">
				<CardHeader>
					<CardTitle>Getting Started</CardTitle>
				</CardHeader>
				<CardContent>
					<p>Get started with one of our quick start guides:</p>
					<div className="flex-1 flex flex-col gap-2 mt-4">
						<div className="flex flex-row justify-stretch items-center gap-2">
							<DocsSheet
								path={docsLinks.gettingStarted.node}
								title="Node.js & Bun Quickstart"
							>
								<Button
									className="flex-1"
									variant="outline"
									startIcon={<Icon icon={faNodeJs} />}
								>
									Node.js & Bun
								</Button>
							</DocsSheet>
							<DocsSheet
								path={docsLinks.gettingStarted.react}
								title="React Quickstart"
							>
								<Button
									className="flex-1"
									variant="outline"
									startIcon={<Icon icon={faReact} />}
								>
									React
								</Button>
							</DocsSheet>
						</div>
					</div>
				</CardContent>
			</Card>

			<Card className="max-w-md w-full mb-6">
				<CardHeader>
					<CardTitle>Connect to Project</CardTitle>
				</CardHeader>
				<CardContent>
					<p className="mb-4">
						Connect to your RivetKit project by entering the URL and
						access token.
					</p>

					<ConnectionForm
						ref={ref}
						defaultValues={{
							username: u || "http://localhost:8080",
							token: token || t || "",
						}}
						onSubmit={async (values, form) => {
							try {
								await queries.getManagerStatus({
									token: values.token,
									url: values.username,
								});
								setToken(values.username, values.token);
								navigate({
									to: ".",
									search: (old) => ({
										...old,
										u: values.username,
										modal: undefined,
									}),
								});
							} catch (error) {
								form.setError("token", {
									message:
										"Failed to connect. Please check your URL and token.",
								});
							}
						}}
					/>
				</CardContent>
			</Card>
		</div>
	);
}
