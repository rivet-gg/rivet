import * as Layout from "@/domains/project/layouts/servers-layout";
import {
	type FunctionInvoke,
	functionsQueryOptions,
} from "@/domains/project/queries";
import {
	Button,
	calculateTableSizing,
	SmallText,
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@rivet-gg/components";
import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
	createColumnHelper,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from "@tanstack/react-table";
import { zodValidator } from "@tanstack/zod-adapter";
import { format } from "date-fns";
import { useLayoutEffect, useRef } from "react";
import { z } from "zod";
import { useVirtualizer } from "@tanstack/react-virtual";
import { faFilter, Icon } from "@rivet-gg/icons";
import { ActorRegion, ActorTags } from "@rivet-gg/components/actors";

const searchSchema = z.object({
	actorId: z.string().optional(),
	tab: z.string().optional(),

	tags: z.array(z.tuple([z.string(), z.string()])).optional(),
	showDestroyed: z.boolean().optional().default(true),
});

export const Route = createFileRoute(
	"/_authenticated/_layout/projects/$projectNameId/environments/$environmentNameId/_v2/functions",
)({
	validateSearch: zodValidator(searchSchema),
	staticData: {
		layout: "v2",
	},
	component: ProjectFunctionsRoute,
	pendingComponent: Layout.Content.Skeleton,
});

const columnHelper = createColumnHelper<FunctionInvoke>();

const columns = [
	columnHelper.accessor("regionId", {
		header: "Region",
		cell: ({ getValue }) => (
			<ActorRegion regionId={getValue()} showLabel="abbreviated" />
		),
	}),
	columnHelper.accessor("id", {
		header: "ID",
		cell: ({ getValue }) => (
			<SmallText>{getValue().split("-")[0]}</SmallText>
		),
	}),
	columnHelper.accessor("timestamp", {
		header: "Time",
		cell: ({ getValue }) => (
			<span className="text-muted-foreground font-mono-console min-w-0 whitespace-nowrap break-keep">
				{format(getValue(), "LLL dd HH:mm:ss O").toUpperCase()}
			</span>
		),
	}),

	columnHelper.accessor("level", {
		header: "Level",
		cell: ({ getValue }) => (
			<span className="text-muted-foreground font-mono-console min-w-0 whitespace-nowrap break-keep">
				{getValue()}
			</span>
		),
	}),
	columnHelper.accessor("message", {
		header: "Message",
		meta: { isGrow: true },
		cell: ({ getValue }) => (
			<span className="text-foreground font-mono-console min-w-0 whitespace-nowrap break-keep">
				{getValue()}
			</span>
		),
	}),
	columnHelper.accessor("properties", {
		header: "Properties",
		cell: ({ getValue }) => <ActorTags tags={getValue()} />,
		meta: { isGrow: true },
	}),
];

function ProjectFunctionsRoute() {
	const { environmentNameId, projectNameId } = Route.useParams();
	const { data } = useSuspenseQuery(
		functionsQueryOptions({ projectNameId, environmentNameId }),
	);

	const table = useReactTable({
		columns,
		data,
		debugTable: true,
		debugHeaders: true,
		debugColumns: true,
		getCoreRowModel: getCoreRowModel(),
	});

	const { rows } = table.getRowModel();

	const tableContainerRef = useRef<HTMLDivElement>(null);

	const virtualizer = useVirtualizer({
		count: rows.length,
		getScrollElement: () => tableContainerRef.current,
		estimateSize: () => 54,
		overscan: 5,
	});

	const headers = table.getFlatHeaders();

	useLayoutEffect(() => {
		if (!tableContainerRef.current) return;
		const resizeObserver = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (entry) {
				const initialColumnSizing = calculateTableSizing(
					headers,
					entry.contentRect.width,
				);
				table.setColumnSizing(initialColumnSizing);
			}
		});
		resizeObserver.observe(tableContainerRef.current);
		return () => {
			resizeObserver.disconnect();
		};
	}, [headers, table]);

	return (
		<div className="flex flex-col max-w-full max-h-full w-full h-full bg-card">
			<div className="flex w-full p-1 py-2 border-b sticky top-0">
				<Button
					size="sm"
					variant="ghost"
					startIcon={<Icon icon={faFilter} />}
				>
					Filters
				</Button>
			</div>
			<div className="flex flex-1 min-h-0 max-h-full">
				<Table containerRef={tableContainerRef}>
					<TableHeader>
						{table.getHeaderGroups().map((headerGroup) => (
							<TableRow key={headerGroup.id}>
								{headerGroup.headers.map((header) => (
									<TableHead
										className="font-semibold text-sm text-foreground"
										key={header.id}
										colSpan={header.colSpan}
										style={{
											width:
												header.index === 4
													? header.getSize()
													: undefined,
										}}
									>
										{header.isPlaceholder
											? null
											: flexRender(
													header.column.columnDef
														.header,
													header.getContext(),
												)}
									</TableHead>
								))}
							</TableRow>
						))}
					</TableHeader>
					<TableBody>
						{virtualizer
							.getVirtualItems()
							.map((virtualRow, index) => {
								const row = rows[virtualRow.index];
								return (
									<TableRow
										key={row.id}
										style={{
											height: `${virtualRow.size}px`,
											transform: `translateY(${
												virtualRow.start -
												index * virtualRow.size
											}px)`,
										}}
										// https://github.com/TanStack/virtual/issues/620
										className="border-b-0 after:absolute after:inset-x-0 after:bottom-0 after:bg-border after:h-px after:contents-[''] py-4"
									>
										{row.getVisibleCells().map((cell) => (
											<TableCell
												key={cell.id}
												style={{
													width: cell.column.getSize(),
												}}
											>
												{flexRender(
													cell.column.columnDef.cell,
													cell.getContext(),
												)}
											</TableCell>
										))}
									</TableRow>
								);
							})}
					</TableBody>
				</Table>
			</div>
		</div>
	);
}
