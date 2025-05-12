import { DatabaseTable } from "@/components/database-table";
import {
	currentTableAtom,
	currentTableColumnsAtom,
	currentTableInfo,
	currentTableRowsAtom,
	dbInfoAtom,
} from "@/stores/db";
import { Button, Checkbox, ScrollArea } from "@rivet-gg/components";
import { ActorsLayout } from "@rivet-gg/components/actors";
import { faTable, Icon } from "@rivet-gg/icons";
import { createFileRoute, Link } from "@tanstack/react-router";
import { createColumnHelper } from "@tanstack/react-table";
import { zodValidator } from "@tanstack/zod-adapter";
import { useAtomValue, useSetAtom } from "jotai";
import { useEffect, useMemo } from "react";
import { z } from "zod";

export const Route = createFileRoute("/_layout/db")({
	validateSearch: zodValidator(
		z.object({
			table: z.string().optional(),
		}),
	),
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<ActorsLayout
			className="h-full min-h-full max-h-full w-full min-w-full max-w-full"
			left={<Sidebar />}
			right={
				<div className="h-full flex flex-1 min-w-0">
					<Content />
				</div>
			}
		/>
	);
}

function Content() {
	const info = useAtomValue(currentTableInfo);
	const remoteColumns = useAtomValue(currentTableColumnsAtom) ?? [];

	const { data: remoteData } = useAtomValue(currentTableRowsAtom);

	return (
		<ScrollArea className="h-full min-w-0 flex-1">
			<DatabaseTable
				columns={remoteColumns}
				data={remoteData ?? []}
				references={info?.references ?? []}
			/>
		</ScrollArea>
	);
}

function Sidebar() {
	const { data } = useAtomValue(dbInfoAtom);

	const { table } = Route.useSearch();

	const setCurrentTable = useSetAtom(currentTableAtom);

	useEffect(() => {
		setCurrentTable(table);
	}, [table, setCurrentTable]);

	return (
		<div className="p-2 flex flex-col justify-between border-r min-w-48">
			<div>
				{/* <div className="h-px mt-4 mb-4 border-t" /> */}
				<p className="text-muted-foreground uppercase text-xs font-semibold my-2">
					Tables
				</p>
				<ul>
					{data?.map(({ records, table }) => {
						return (
							<li key={table.name} className="w-full">
								<Button
									variant="ghost"
									startIcon={<Icon icon={faTable} />}
									endIcon={
										<span className="text-muted-foreground text-xs">
											{records}
										</span>
									}
									asChild
									className="w-full text-left justify-start data-active:bg-secondary data-active:text-primary-foreground data-active:font-semibold"
								>
									<Link to="." search={{ table: table.name }}>
										<span className="flex-1">
											{table.name}
										</span>
									</Link>
								</Button>
							</li>
						);
					})}
				</ul>
			</div>
		</div>
	);
}
