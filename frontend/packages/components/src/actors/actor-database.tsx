import { useState } from "react";
import type { ActorId } from "./queries";
import { useQuery } from "@tanstack/react-query";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "../ui/select";
import { Icon, faRefresh, faTable, faTableCells } from "@rivet-gg/icons";
import { Flex } from "../ui/flex";
import { DatabaseTable } from "./database/database-table";
import { ScrollArea } from "../ui/scroll-area";
import { Button } from "../ui/button";
import { WithTooltip } from "../ui/tooltip";
import { ShimmerLine } from "../shimmer-line";
import { useActorQueries } from "./actor-queries-context";

interface ActorDatabaseProps {
	actorId: ActorId;
}

export function ActorDatabase({ actorId }: ActorDatabaseProps) {
	const actorQueries = useActorQueries();
	const { data, refetch } = useQuery(
		actorQueries.actorDatabaseQueryOptions(actorId),
	);
	const [table, setTable] = useState<string | undefined>(
		() => data?.db[0]?.table.name,
	);

	const selectedTable = table || data?.db[0]?.table.name;

	const {
		data: rows,
		refetch: refetchData,
		isLoading,
	} = useQuery(
		actorQueries.actorDatabaseRowsQueryOptions(actorId, selectedTable!, {
			enabled: !!selectedTable,
		}),
	);

	const currentTable = data?.db.find((db) => db.table.name === selectedTable);

	return (
		<>
			<div className="flex justify-between items-center border-b gap-1 h-[45px]">
				<div className="border-r h-full ">
					<TableSelect
						actorId={actorId}
						onSelect={setTable}
						value={table}
					/>
				</div>
				<div className="flex-1 text-xs">
					<Flex className="items-center gap-2 h-full px-2">
						<Icon icon={faTableCells} />
						{currentTable ? (
							<>
								{currentTable.table.schema}.
								{currentTable.table.name}
								<span className="text-muted-foreground">
									({currentTable.columns.length} columns,{" "}
									{currentTable.records} rows)
								</span>
							</>
						) : (
							<span className="text-muted-foreground">
								No table selected
							</span>
						)}
					</Flex>
				</div>
				<div className="border-l h-full flex items-center gap-2 px-2">
					<WithTooltip
						content="Refresh"
						trigger={
							<Button
								variant="ghost"
								size="icon-sm"
								isLoading={isLoading}
								onClick={() => {
									refetch();
									refetchData();
								}}
							>
								<Icon icon={faRefresh} />
							</Button>
						}
					/>
				</div>
			</div>
			<div className="flex-1 min-h-0 overflow-hidden flex relative">
				{isLoading ? <ShimmerLine /> : null}
				<ScrollArea className="w-full h-full min-h-0">
					{currentTable ? (
						<DatabaseTable
							className="overflow-hidden"
							columns={currentTable?.columns}
							enableColumnResizing={false}
							enableRowSelection={false}
							data={
								rows && "result" in rows
									? (rows.result as unknown[])
									: []
							}
							references={currentTable?.foreignKeys}
						/>
					) : null}
				</ScrollArea>
			</div>
		</>
	);
}

function TableSelect({
	actorId,
	value,
	onSelect,
}: {
	actorId: ActorId;
	onSelect: (table: string) => void;
	value: string | undefined;
}) {
	const actorQueries = useActorQueries();
	const { data: tables } = useQuery(
		actorQueries.actorDatabaseTablesQueryOptions(actorId),
	);

	return (
		<Select onValueChange={onSelect} value={value}>
			<SelectTrigger variant="ghost" className="h-full pr-2 rounded-none">
				<SelectValue placeholder="Select table..." />
			</SelectTrigger>
			<SelectContent>
				{tables?.map((table) => (
					<SelectItem key={table.name} value={table.name}>
						<div className="flex items-center gap-2">
							<Icon icon={faTable} className="text-foreground" />
							{table.name}
						</div>
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
}
