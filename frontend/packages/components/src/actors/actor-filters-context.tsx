import {
	faTag,
	faCalendarCirclePlus,
	faCalendarCircleMinus,
	faSignalBars,
	faGlobe,
	faCode,
} from "@rivet-gg/icons";
import { createContext, useContext } from "react";
import {
	FilterOp,
	type FilterDefinitions,
	createFiltersPicker,
	createFiltersRemover,
	createFiltersSchema,
	type OptionsProviderProps,
} from "../ui/filters";
import { ActorRegion } from "./actor-region";
import { ActorStatus } from "./actor-status";
import { useQuery, useInfiniteQuery } from "@tanstack/react-query";
import { CommandGroup, CommandItem } from "cmdk";
import { cn } from "../lib/utils";
import { ActorTag } from "./actor-tags";
import { useManagerQueries } from "./manager-queries-context";
import type { ActorStatus as ActorStatusType } from "./queries";
import { Checkbox } from "../ui/checkbox";

export const ACTORS_FILTERS_DEFINITIONS = {
	tags: {
		type: "select",
		label: "Tags",
		icon: faTag,
		options: TagsOptions,
		operators: {
			[FilterOp.EQUAL]: "is one of",
			[FilterOp.NOT_EQUAL]: "is not one of",
		},
	},
	createdAt: {
		type: "date",
		label: "Created",
		icon: faCalendarCirclePlus,
	},
	destroyedAt: {
		type: "date",
		label: "Destroyed",
		icon: faCalendarCircleMinus,
	},
	status: {
		type: "select",
		label: "Status",
		icon: faSignalBars,
		options: StatusOptions,
		display: ({ value }) => {
			if (value.length > 1) {
				return <span>{value.length} statuses</span>;
			}
			return (
				<ActorStatus
					className="border-0 p-0"
					status={value[0] as ActorStatusType}
				/>
			);
		},
	},
	region: {
		type: "select",
		label: "Region",
		icon: faGlobe,
		options: RegionOptions,
		display: ({ value }) => {
			if (value.length > 1) {
				return <span>{value.length} regions</span>;
			}

			return <ActorRegion regionId={value[0]} showLabel />;
		},
		operators: {
			[FilterOp.EQUAL]: "is one of",
			[FilterOp.NOT_EQUAL]: "is not one of",
		},
	},
	devMode: {
		type: "boolean",
		label: "Show hidden actors",
		icon: faCode,
	},
} satisfies FilterDefinitions;

const defaultActorFiltersContextValue = {
	definitions: ACTORS_FILTERS_DEFINITIONS,
	get pick() {
		return createFiltersPicker(this.definitions);
	},
	get schema() {
		return createFiltersSchema(this.definitions);
	},
	get remove() {
		return createFiltersRemover(this.definitions);
	},
};

export const ActorsFilters = createContext(defaultActorFiltersContextValue);

export const ActorsFiltersProvider = ActorsFilters.Provider;

export const useActorsFilters = () => {
	const context = useContext(ActorsFilters);
	if (!context) {
		throw new Error("useActorsFilters must be used within ActorsFilters");
	}
	return context;
};

function StatusOptions({ onSelect, value: filterValue }: OptionsProviderProps) {
	return (
		<CommandGroup>
			{["running", "starting", "crashed", "stopped"].map((key) => {
				const isSelected = filterValue.some((val) => val === key);
				return (
					<CommandItem
						key={key}
						className="group flex gap-2 items-center px-3 py-1"
						value={key}
						onSelect={() => {
							if (isSelected) {
								onSelect(
									filterValue.filter(
										(filterKey) => filterKey !== key,
									),
									{ closeAfter: true },
								);
								return;
							}

							onSelect([...filterValue, key], {
								closeAfter: true,
							});
						}}
					>
						<Checkbox
							checked={isSelected}
							className={cn({
								"opacity-0 group-data-[selected=true]:opacity-100":
									!isSelected,
							})}
						/>
						<ActorStatus status={key as ActorStatusType} />
					</CommandItem>
				);
			})}
		</CommandGroup>
	);
}

function RegionOptions({ onSelect, value: filterValue }: OptionsProviderProps) {
	const { data: regions = [] } = useQuery(
		useManagerQueries().regionsQueryOptions(),
	);
	return (
		<CommandGroup>
			{regions.map(({ id, name }) => {
				const isSelected = filterValue.some((val) => val === id);
				return (
					<CommandItem
						key={id}
						className="group flex gap-2 items-center px-3 py-1"
						value={id}
						onSelect={() => {
							if (isSelected) {
								onSelect(
									filterValue.filter(
										(filterKey) => filterKey !== id,
									),
									{ closeAfter: true },
								);
								return;
							}

							onSelect([...filterValue, id], {
								closeAfter: true,
							});
						}}
					>
						<Checkbox
							checked={isSelected}
							className={cn({
								"opacity-0 group-data-[selected=true]:opacity-100":
									!isSelected,
							})}
						/>
						<ActorRegion regionId={id} showLabel />
					</CommandItem>
				);
			})}
		</CommandGroup>
	);
}

function TagsOptions({ onSelect, value: filterValue }: OptionsProviderProps) {
	const { data: tags = [] } = useInfiniteQuery(
		useManagerQueries().actorsTagsQueryOptions(),
	);

	const values = filterValue.map((filter) => filter.split("="));

	return (
		<CommandGroup>
			{tags.map(({ key, value }) => {
				const isSelected = values.some(
					([filterKey, filterValue]) =>
						filterKey === key && filterValue === value,
				);
				return (
					<CommandItem
						key={`${key}-${value}`}
						className="group flex gap-2 items-center px-3 py-1"
						value={`${key}=${value}`}
						onSelect={() => {
							if (isSelected) {
								onSelect(
									values
										.filter(
											([filterKey, filterValue]) =>
												filterKey !== key ||
												filterValue !== value,
										)
										.map((pair) => pair.join("=")),
									{ closeAfter: true },
								);
								return;
							}
							onSelect([...filterValue, `${key}=${value}`], {
								closeAfter: true,
							});
						}}
					>
						<Checkbox
							checked={isSelected}
							className={cn({
								"opacity-0 group-data-[selected=true]:opacity-100":
									!isSelected,
							})}
						/>
						<ActorTag className="text-foreground">
							<span className="break-all">
								{key}={value}
							</span>
						</ActorTag>
					</CommandItem>
				);
			})}
		</CommandGroup>
	);
}
