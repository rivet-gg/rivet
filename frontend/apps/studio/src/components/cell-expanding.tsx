import {
	type Cell,
	type Column,
	functionalUpdate,
	makeStateUpdater,
	type OnChangeFn,
	type Row,
	type RowData,
	type Table,
	type TableFeature,
	type Updater,
} from "@tanstack/react-table";

export type ExpandedCellState = Record<string, Record<string, boolean>>;
export interface ExpandedCellTableState {
	expandedCells: ExpandedCellState;
}
export interface ExpandedCellOptions<TData extends RowData> {
	onExpandedCellsChange?: OnChangeFn<ExpandedCellState>;
	enableCellExpanding?: boolean;
	getCellCanExpand?: (row: Row<TData>, cell: Cell<TData, any>) => boolean;
}
export interface ExpandedCellInstance<TData extends RowData> {
	setCellExpanded: (updater: Updater<ExpandedCellState>) => void;
}
export interface ExpandedCellRow<TData extends RowData> {
	getIsSomeCellExpanded: () => boolean;
	getExpandedCell: () => Cell<unknown, any> | undefined;
}
export interface ExpandedCell<TData, TValue> {
	getToggleExpandedHandler: () => () => void;
	getIsExpanded: () => boolean;
	getCanExpand: () => boolean;
	toggleExpanded: (expanded?: boolean) => void;
}

declare module "@tanstack/react-table" {
	//merge our new feature's state with the existing table state
	interface TableState extends ExpandedCellTableState {}
	//merge our new feature's options with the existing table options
	interface TableOptionsResolved<TData extends RowData>
		extends ExpandedCellOptions<TData> {}
	//merge our new feature's instance APIs with the existing table instance APIs
	interface Table<TData extends RowData>
		extends ExpandedCellInstance<TData> {}
	// if you need to add cell instance APIs...
	interface Cell<TData, TValue> extends ExpandedCell<TData, TValue> {}
	// if you need to add row instance APIs...
	interface Row<TData extends RowData> extends ExpandedCellRow<RowData> {}
	// if you need to add column instance APIs...
	// interface Column<TData extends RowData, TValue> extends DensityColumn
	// if you need to add header instance APIs...
	// interface Header<TData extends RowData, TValue> extends DensityHeader

	// Note: declaration merging on `ColumnDef` is not possible because it is a type, not an interface.
	// But you can still use declaration merging on `ColumnDef.meta`
}

// Here is all of the actual javascript code for our new feature
export const CellExpanding: TableFeature<any> = {
	// define the new feature's initial state
	getInitialState: (state): ExpandedCellTableState => {
		return {
			expandedCells: {},
			...state,
		};
	},

	// define the new feature's default options
	getDefaultOptions: <TData extends RowData>(
		table: Table<TData>,
	): ExpandedCellOptions<TData> => {
		return {
			enableCellExpanding: true,
			onExpandedCellsChange: makeStateUpdater("expandedCells", table),
		};
	},

	createTable: <TData extends RowData>(table: Table<TData>): void => {
		table.setCellExpanded = (updater) => {
			const safeUpdater: Updater<ExpandedCellState> = (old) => {
				const newState = functionalUpdate(updater, old);
				return newState;
			};
			return table.options.onExpandedCellsChange?.(safeUpdater);
		};
	},

	createRow: <TData extends RowData>(
		row: Row<TData>,
		table: Table<TData>,
	): void => {
		row.getIsSomeCellExpanded = () => {
			const expanded = table.getState().expandedCells;
			return !!expanded?.[row.id];
		};
		row.getExpandedCell = (() => {
			const expanded = table.getState().expandedCells;
			const cellIds = Object.keys(expanded?.[row.id]);
			return row.getAllCells().find((cell) => cellIds.includes(cell.id));
		}) as ExpandedCellRow<TData>["getExpandedCell"];
	},

	createCell: <TData extends RowData, TValue>(
		cell: Cell<TData, TValue>,
		column: Column<TData>,
		row: Row<TData>,
		table: Table<TData>,
	): void => {
		cell.getIsExpanded = () => {
			return !!table.getState().expandedCells[row.id]?.[cell.id];
		};
		cell.toggleExpanded = (expanded) => {
			row.toggleExpanded(expanded);
			table.setCellExpanded((old) => {
				const exists = !!old?.[row.id]?.[cell.id];

				const oldExpanded: ExpandedCellState = old;
				const newValue = expanded ?? !exists;

				if (!exists && newValue) {
					return {
						...oldExpanded,
						[row.id]: {
							[cell.id]: true,
						},
					};
				}

				if (exists && !newValue) {
					return {
						...oldExpanded,
						[row.id]: {
							...(oldExpanded?.[row.id] ?? {}),
							[cell.id]: false,
						},
					};
				}

				return old;
			});
		};
		cell.getToggleExpandedHandler = () => {
			return () => cell.toggleExpanded();
		};
		cell.getCanExpand = () => {
			return (
				table.options.getCellCanExpand?.(row, cell) ??
				table.options.enableCellExpanding ??
				true
			);
		};
	},
	// if you need to add column instance APIs...
	// createColumn: <TData extends RowData>(column, table): void => {},
	// if you need to add header instance APIs...
	// createHeader: <TData extends RowData>(header, table): void => {},
};
