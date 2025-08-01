import {
	Button,
	DocsSheet,
	FilterCreator,
	type OnFiltersChange,
	ScrollArea,
	ShimmerLine,
	SmallText,
} from "@rivet-gg/components";
import { ActorsListRow } from "./actors-list-row";
import { CreateActorButton } from "./create-actor-button";
import { GoToActorButton } from "./go-to-actor-button";
import { useSearch, useNavigate } from "@tanstack/react-router";
import {
	Icon,
	faActors,
	faCalendarMinus,
	faCalendarPlus,
	faGlobe,
	faNodeJs,
	faReact,
} from "@rivet-gg/icons";
import { useCallback, useMemo } from "react";
import { useInfiniteQuery } from "@tanstack/react-query";
import { useActorsView } from "./actors-view-context-provider";
import { useManagerQueries } from "./manager-queries-context";
import { useActorsFilters } from "./actor-filters-context";

export function ActorsList() {
	return (
		<>
			<ScrollArea className="w-full @container/main">
				<div className="grid grid-cols-[2rem_1rem_1fr_1fr_1fr_1fr] @lg/main:grid-cols-[2rem_min-content_min-content_minmax(1rem,2fr)_minmax(min-content,1fr)_minmax(min-content,1fr)] items-center justify-center gap-x-4 w-full min-w-[450px] @container/table">
					<div className="grid grid-cols-subgrid col-span-full sticky top-0 z-[1] bg-card">
						<div className="col-span-full border-b justify-between flex px-2 py-2 gap-1 relative h-[45px]">
							{/* <Filters /> */}
							<div className="flex gap-1">
								<GoToActorButton />
								<CreateActorButton />
							</div>
							<LoadingIndicator />
						</div>
						<div className="grid grid-cols-subgrid col-span-full font-semibold text-sm px-1 pr-4 h-[45px] items-center  border-b">
							<div />
							<div>
								<span className="hidden  @[500px]/table:inline">
									Region
								</span>
								<span className="@[500px]/table:hidden">
									<Icon icon={faGlobe} />
								</span>
							</div>
							<div>ID</div>
							<div>Tags</div>
							<div>
								<span className="hidden @[500px]/table:inline">
									Created
								</span>
								<span className="@[500px]/table:hidden">
									<Icon icon={faCalendarPlus} />
								</span>
							</div>
							<div>
								<span className="hidden @[500px]/table:inline">
									Destroyed
								</span>
								<span className="@[500px]/table:hidden">
									<Icon icon={faCalendarMinus} />
								</span>
							</div>
						</div>
					</div>
					<List />
					<Pagination />
				</div>
			</ScrollArea>
		</>
	);
}

function LoadingIndicator() {
	const { isLoading } = useInfiniteQuery(
		useManagerQueries().actorsListQueryOptions(),
	);
	if (isLoading) {
		return <ShimmerLine className="bottom-0" />;
	}
	return null;
}

function List() {
	const { data: actorIds = [] } = useInfiniteQuery(
		useManagerQueries().actorsListQueryOptions(),
	);

	const actorId = useSearch({ select: (state) => state.actorId });

	return (
		<>
			{actorIds.map((id) => (
				<ActorsListRow
					key={id}
					actorId={id}
					isCurrent={actorId === id}
				/>
			))}
		</>
	);
}

function Pagination() {
	const { hasNextPage, isFetchingNextPage, fetchNextPage, data } =
		useInfiniteQuery(
			useManagerQueries().actorsListPaginationQueryOptions(),
		);

	if (hasNextPage) {
		return (
			<div className="col-span-full flex w-full justify-center py-4">
				<Button
					variant="outline"
					mx="4"
					isLoading={isFetchingNextPage}
					onClick={() => fetchNextPage()}
				>
					Load more
				</Button>
			</div>
		);
	}

	return <EmptyState count={data || 0} />;
}

function EmptyState({ count }: { count: number }) {
	const navigate = useNavigate();
	const { copy, links } = useActorsView();
	const { remove, pick } = useActorsFilters();

	const filtersCount = useSearch({
		select: (state) => Object.values(pick(state)).length,
	});

	const clearFilters = () => {
		navigate({
			search: (prev) => ({
				...remove(prev),
			}),
		});
	};

	return (
		<div className=" col-span-full my-4 flex flex-col items-center gap-2 justify-center">
			{count === 0 ? (
				filtersCount === 0 ? (
					<div className="gap-2 flex flex-col items-center justify-center">
						<Icon icon={faActors} className="text-4xl mb-2 mt-8" />
						<SmallText className="text-center">
							{copy.noActorsFound}
						</SmallText>
						<div className="mt-4 flex flex-col gap-2 items-center justify-center">
							<CreateActorButton variant="secondary" />{" "}
							<SmallText className="mt-4 mb-1">
								Use one of the quick start guides to get
								started.
							</SmallText>
							<div className="flex gap-2">
								<DocsSheet
									path={links.gettingStarted.node}
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
									path={links.gettingStarted.react}
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
					</div>
				) : (
					<>
						<SmallText className="text-foreground text-center mt-8 mb-2">
							{copy.noActorsMatchFilter}
						</SmallText>
						<Button variant="outline" mx="4" onClick={clearFilters}>
							Clear filters
						</Button>
					</>
				)
			) : (
				<SmallText className="text-foreground text-center">
					{copy.noMoreActors}
				</SmallText>
			)}
		</div>
	);
}

function Filters() {
	const navigate = useNavigate();
	const filters = useSearch({ strict: false });

	const { pick, remove } = useActorsFilters();

	const onFiltersChange: OnFiltersChange = useCallback(
		(fnOrValue) => {
			if (typeof fnOrValue === "function") {
				navigate({
					search: (old) => {
						const filters = pick(old);
						const prev = remove(old);

						return {
							...prev,
							...Object.fromEntries(
								Object.entries(fnOrValue(filters)).filter(
									([, filter]) => filter.value.length > 0,
								),
							),
						};
					},
				});
			} else {
				navigate({
					search: (value) => ({
						...remove(value),
						...Object.fromEntries(
							Object.entries(fnOrValue).filter(
								([, filter]) => filter.value.length > 0,
							),
						),
					}),
				});
			}
		},
		[navigate, pick],
	);

	const { copy } = useActorsView();

	const { definitions } = useActorsFilters();

	const filtersDefs = useMemo(() => {
		return {
			...definitions,
			devMode: {
				...definitions.devMode,
				hidden: true,
				label: copy.showHiddenActors,
			},
		};
	}, [copy.showHiddenActors, definitions]);

	return (
		<FilterCreator
			value={filters}
			onChange={onFiltersChange}
			definitions={filtersDefs}
		/>
	);
}
