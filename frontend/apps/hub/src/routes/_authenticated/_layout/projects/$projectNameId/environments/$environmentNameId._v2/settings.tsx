import { ErrorComponent } from "@/components/error-component";
import { useEnvironment } from "@/domains/project/data/environment-context";
import { useProject } from "@/domains/project/data/project-context";
import * as Layout from "@/domains/project/layouts/project-layout";
import {
	routesQueryOptions,
	useDeleteRouteMutation,
} from "@/domains/project/queries";
import { useDialog } from "@/hooks/use-dialog";
import {
	ActionCard,
	Button,
	Card,
	CardContent,
	CardHeader,
	CardTitle,
	DiscreteCopyButton,
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
	Flex,
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
	Text,
	WithTooltip,
} from "@rivet-gg/components";
import { Icon, faEllipsisH, faPlus } from "@rivet-gg/icons";
import { useSuspenseQuery } from "@tanstack/react-query";
import {
	type ErrorComponentProps,
	Link,
	createFileRoute,
} from "@tanstack/react-router";
import { zodValidator } from "@tanstack/zod-adapter";
import { z } from "zod";

function EnvironmentSettingsRoute() {
	return (
		<div className="p-4">
			<div className="max-w-5xl mx-auto flex flex-col gap-8">
				<RoutesCard />
				<ServiceTokenCard />
				<Modals />
			</div>
		</div>
	);
}

function ServiceTokenCard() {
	return (
		<>
			<ActionCard
				title="Service token"
				footer={
					<Button asChild>
						<Link to="." search={{ modal: "service-token" }}>
							Generate
						</Link>
					</Button>
				}
			>
				<Text>
					Service tokens are used from private API servers. These
					should never be shared.
				</Text>
			</ActionCard>
		</>
	);
}

function RoutesCard() {
	const { projectNameId, environmentNameId } = Route.useParams();
	const { data: routes } = useSuspenseQuery(
		routesQueryOptions(Route.useParams()),
	);
	const navigate = Route.useNavigate();

	const { mutate: deleteRoute } = useDeleteRouteMutation();

	return (
		<Card w="full">
			<CardHeader>
				<Flex items="center" gap="4" justify="between">
					<CardTitle>Routes</CardTitle>
					<div className="flex gap-2">
						<Button
							variant="outline"
							startIcon={<Icon icon={faPlus} />}
							asChild
						>
							<Link
								to="."
								search={{ modal: "add-route" }}
								params={{ projectNameId, environmentNameId }}
							>
								Add Route
							</Link>
						</Button>
					</div>
				</Flex>
			</CardHeader>
			<CardContent>
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead>ID</TableHead>
							<TableHead>Route</TableHead>
							<TableHead>Instances</TableHead>
							<TableHead />
						</TableRow>
					</TableHeader>
					<TableBody>
						{routes.length === 0 ? (
							<TableRow>
								<TableCell colSpan={4}>
									<Text className="text-center">
										There's no routes yet.
									</Text>
								</TableCell>
							</TableRow>
						) : null}
						{routes?.map((route) => (
							<TableRow key={route.id}>
								<TableCell>
									<DiscreteCopyButton value={route.nameId}>
										{route.nameId}
									</DiscreteCopyButton>
								</TableCell>
								<TableCell>
									<DiscreteCopyButton
										value={`${route.hostname}${route.path}${route.routeSubpaths ? "/*" : ""}`}
									>
										{`${route.hostname}${route.path}${route.routeSubpaths ? "/*" : ""}`}
									</DiscreteCopyButton>
								</TableCell>
								<TableCell>
									<WithTooltip
										content="Click to view instances"
										trigger={20}
									/>
								</TableCell>
								<TableCell>
									<DropdownMenu>
										<DropdownMenuTrigger asChild>
											<Button
												aria-haspopup="true"
												size="icon"
												variant="ghost"
											>
												<Icon
													className="size-4"
													icon={faEllipsisH}
												/>
												<span className="sr-only">
													Toggle menu
												</span>
											</Button>
										</DropdownMenuTrigger>
										<DropdownMenuContent align="end">
											<DropdownMenuItem
												onSelect={() =>
													navigate({
														to: ".",
														search: {
															modal: "edit-route",
															route: route.nameId,
														},
														params: {
															projectNameId,
															environmentNameId,
														},
													})
												}
											>
												Edit
											</DropdownMenuItem>
											<DropdownMenuItem
												onSelect={() => {
													deleteRoute({
														projectNameId,
														environmentNameId,
														routeNameId:
															route.nameId,
													});
												}}
											>
												Delete
											</DropdownMenuItem>
										</DropdownMenuContent>
									</DropdownMenu>
								</TableCell>
							</TableRow>
						))}
					</TableBody>
				</Table>
			</CardContent>
		</Card>
	);
}

function Modals() {
	const navigate = Route.useNavigate();
	const { gameId: projectId, nameId: projectNameId } = useProject();
	const { namespaceId: environmentId, nameId: environmentNameId } =
		useEnvironment();

	const { modal, route } = Route.useSearch();

	const GenerateProjectEnvServiceTokenDialog =
		useDialog.GenerateProjectEnvServiceToken.Dialog;

	const EditRouteDialog = useDialog.EditRoute.Dialog;
	const CreateRouteDialog = useDialog.CreateRoute.Dialog;

	const handleOnOpenChange = (value: boolean) => {
		if (!value) {
			navigate({ search: { modal: undefined } });
		}
	};

	return (
		<>
			<GenerateProjectEnvServiceTokenDialog
				projectId={projectId}
				environmentId={environmentId}
				dialogProps={{
					open: modal === "service-token",
					onOpenChange: handleOnOpenChange,
				}}
			/>
			<EditRouteDialog
				projectNameId={projectNameId}
				environmentNameId={environmentNameId}
				// biome-ignore lint/style/noNonNullAssertion: at this point this should exist
				routeNameId={route!}
				dialogProps={{
					open: modal === "edit-route" && route !== undefined,
					onOpenChange: handleOnOpenChange,
				}}
			/>
			<CreateRouteDialog
				projectNameId={projectNameId}
				environmentNameId={environmentNameId}
				dialogProps={{
					open: modal === "add-route",
					onOpenChange: handleOnOpenChange,
				}}
			/>
		</>
	);
}

const searchSchema = z.object({
	modal: z
		.enum(["service-token", "add-route", "edit-route"])
		.or(z.string())
		.optional(),
	route: z.string().optional(),
});

export const Route = createFileRoute(
	"/_authenticated/_layout/projects/$projectNameId/environments/$environmentNameId/_v2/settings",
)({
	staticData: {
		layout: "v2",
	},
	validateSearch: zodValidator(searchSchema),
	component: EnvironmentSettingsRoute,
	pendingComponent: () => (
		<div className="flex flex-col gap-4 p-4">
			<Layout.Root.Skeleton />
		</div>
	),
	errorComponent(props: ErrorComponentProps) {
		return (
			<div className="p-4">
				<div className="max-w-5xl mx-auto">
					<ErrorComponent {...props} />
				</div>
			</div>
		);
	},
});
