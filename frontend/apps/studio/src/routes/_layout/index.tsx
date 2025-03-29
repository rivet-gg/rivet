import { Actors } from "@/components/actors";
import {
	ACTOR_CORE_MANAGER_PORT,
	connectionEffect,
	connectionStateAtom,
	initiallyConnectedAtom,
} from "@/stores/manager";
import {
	Button,
	Card,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
	Code,
	CodeFrame,
	CodeGroup,
	CodeSource,
	DocsSheet,
	Link,
	Strong,
	Text,
} from "@rivet-gg/components";
import { currentActorIdAtom } from "@rivet-gg/components/actors";
import {
	Icon,
	faActors,
	faBrave,
	faReact,
	faRust,
	faSafari,
	faTs,
} from "@rivet-gg/icons";
import { createFileRoute } from "@tanstack/react-router";
import { zodValidator } from "@tanstack/zod-adapter";
import { AnimatePresence, motion } from "framer-motion";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import { useEffect } from "react";
import { z } from "zod";
// @ts-expect-error types are missing
import devNpm, { source as devNpmSource } from "../../content/dev-npm.sh?shiki";
import devYarn, {
	source as devYarnSource,
	// @ts-expect-error types are missing
} from "../../content/dev-yarn.sh?shiki";
import devPnpm, {
	source as devPnpmSource,
	// @ts-expect-error types are missing
} from "../../content/dev-pnpm.sh?shiki";

export const Route = createFileRoute("/_layout/")({
	component: RouteComponent,
	validateSearch: zodValidator(
		z.object({
			actorId: z.string().optional(),
			tab: z.string().optional(),
			// filters
			tags: z.array(z.tuple([z.string(), z.string()])).optional(),
			showDestroyed: z.boolean().optional(),
		}),
	),
});

function RouteComponent() {
	useAtom(connectionEffect);

	const isInitiallyConnected = useAtomValue(initiallyConnectedAtom);
	const status = useAtomValue(connectionStateAtom);

	const { actorId } = Route.useSearch();

	const setCurrentActorId = useSetAtom(currentActorIdAtom);

	useEffect(() => {
		setCurrentActorId(actorId);
	}, [actorId, setCurrentActorId]);

	return (
		<AnimatePresence initial={false}>
			{status === "disconnected" && !isInitiallyConnected ? (
				<motion.div
					initial={{ opacity: 0, scale: 0.95 }}
					animate={{ opacity: 1, scale: 1 }}
					exit={{ opacity: 0, scale: 0.95 }}
					className="h-full w-full flex items-center justify-center flex-col"
				>
					<div className="max-w-md">
						<Card>
							<CardHeader className="flex flex-row items-center justify-start gap-4">
								<Icon icon={faActors} className="text-6xl" />
								<div className="flex flex-col gap-1">
									<CardTitle>Rivet Studio</CardTitle>
									<CardDescription>
										Take your Actors to the next level
									</CardDescription>
								</div>
							</CardHeader>
							<CardContent>
								<Text>
									<Strong>Welcome!</Strong>
								</Text>
								<Text>
									To get started, you need to connect to your
									ActorCore project.
								</Text>

								<div className="my-4 flex flex-col gap-2">
									<Text className="text-sm flex-1">
										<span className="font-bold">
											Don't have an ActorCore project yet?
										</span>
										<br />
										Get started with one of our quick start
										guides.
									</Text>
									<div className="flex-1 flex flex-col gap-2">
										<div className="flex flex-row justify-stretch items-center gap-0.5">
											<DocsSheet
												path="docs/quickstart/react"
												title="React Quick Start"
											>
												<Button
													className="flex-1"
													variant="outline"
													size="sm"
													startIcon={
														<Icon icon={faReact} />
													}
												>
													React
												</Button>
											</DocsSheet>
											<DocsSheet
												path="docs/quickstart/typescript"
												title="TypeScript Quick Start"
											>
												<Button
													className="flex-1"
													variant="outline"
													size="sm"
													startIcon={
														<Icon icon={faTs} />
													}
												>
													TypeScript
												</Button>
											</DocsSheet>
											<DocsSheet
												path="docs/quickstart/typescript"
												title="Rust Quick Start"
											>
												<Button
													className="flex-1"
													variant="outline"
													size="sm"
													startIcon={
														<Icon icon={faRust} />
													}
												>
													Rust
												</Button>
											</DocsSheet>
										</div>
									</div>
								</div>

								<hr className="my-6" />

								<Text>
									To connect, make sure you have the ActorCore
									CLI installed and running. You can do this
									by running the following command in your
									project directory.
								</Text>

								<CodeGroup>
									<CodeFrame
										title="npm"
										language="bash"
										code={devNpmSource}
									>
										<CodeSource>{devNpm}</CodeSource>
									</CodeFrame>
									<CodeFrame
										title="yarn"
										language="bash"
										code={devYarnSource}
									>
										<CodeSource>{devYarn}</CodeSource>
									</CodeFrame>
									<CodeFrame
										title="pnpm"
										language="bash"
										code={devPnpmSource}
									>
										<CodeSource>{devPnpm}</CodeSource>
									</CodeFrame>
								</CodeGroup>

								<Text>
									This will start the development server and
									allow Rivet Studio to connect to your
									project.
								</Text>
								<Text>
									The Studio will automatically connect to the
									ActorCore development server running on{" "}
									<Code>
										localhost:{ACTOR_CORE_MANAGER_PORT}
									</Code>
									. It will also automatically detect changes,
									and reconnect when you restart the server.
								</Text>

								<div className="gap-2 flex flex-col mt-6">
									<p className="my-0 text-sm font-bold">
										Having trouble connecting?
									</p>
									<p className="my-0 text-sm">
										Make sure that your browser allow access
										to localhost. Some browsers like{" "}
										<Strong>
											<Icon icon={faSafari} /> Safari
										</Strong>{" "}
										and{" "}
										<Strong>
											<Icon icon={faBrave} /> Brave
										</Strong>{" "}
										block access by default.
									</p>
								</div>
							</CardContent>
							<CardFooter>
								<div className="text-muted-foreground text-xs">
									Need help? Join the{" "}
									<Link
										href="https://rivet.gg/discord"
										target="_blank"
										rel="noopener noreferrer"
									>
										Rivet Discord
									</Link>
									.<br /> Looking for docs? Check out the{" "}
									<Link
										href="https://actorcore.org/docs"
										target="_blank"
										rel="noopener noreferrer"
									>
										ActorCore documentation
									</Link>
									.
								</div>
							</CardFooter>
						</Card>
					</div>
				</motion.div>
			) : (
				<Actors actorId={actorId} />
			)}
		</AnimatePresence>
	);
}
