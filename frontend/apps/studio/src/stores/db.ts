import { queryOptions } from "@tanstack/react-query";
import { atom } from "jotai";
import { atomWithQuery } from "jotai-tanstack-query";
import { atomFamily, splitAtom } from "jotai/utils";

export const dbInfoAtom = atomWithQuery(() => ({
	queryKey: ["db", "info"],
	queryFn: async () => {
		const res = await fetch(
			"http://localhost:3456/tables"
		);
        if(!res.ok) {
            throw new Error("Network response was not ok");
        }
		return res.json();
	},
}));

export const currentTableAtom = atom<string | null>(null);
export const currentTableInfo = atom((get) => {
    const dbInfo = get(dbInfoAtom);
    const currentTable = get(currentTableAtom);
    return dbInfo?.data?.find(({table}) => table.name === currentTable);
})
export const currentTableColumnsAtom = atom((get) => {
    const dbInfo = get(dbInfoAtom);
    const currentTable = get(currentTableAtom);
    return dbInfo?.data?.find(({table}) => table.name === currentTable)?.columns;
})

export const tablesAtom = atom((get) => get(dbInfoAtom).data)
export const tableInfo = splitAtom(tablesAtom, (table) => table.name);

export const currentTableRowsAtom = atomWithQuery((get) => ({
	queryKey: ["db", get(currentTableAtom), "rows"],
	queryFn: async ({queryKey: [,currentTable]}) => {
		const res = await fetch(
			"http://localhost:3456/query",
            {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    // FIXME
                    sql: `SELECT * FROM ${currentTable} LIMIT 500`,
                    args: [],
                    // args: [get(currentTableAtom)],
                }),
            }
		);
        if(!res.ok) {
            throw new Error("Network response was not ok");
        }
		return res.json();
	},
}));

export const tableReferenceRowsQueryOptions = (table: string, opts: {values: any[], property: string}) => queryOptions({
    queryKey: ["db", table, "rows", opts] as const,
    queryFn: async ({queryKey: [,table, ,opts]}) => {
        const res = await fetch(
            "http://localhost:3456/query",
            {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    sql: `SELECT * FROM ${table} WHERE (${opts.property}) IN (${opts.values.join(",")})`,
                    args: [],
                }),
            }
        );
        if(!res.ok) {
            throw new Error("Network response was not ok");
        }
        return res.json();
    },
});