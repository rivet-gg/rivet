import { VirtualScrollArea } from "@rivet-gg/components";
import { useQueries } from "@tanstack/react-query";
import type { Virtualizer } from "@tanstack/react-virtual";
import { memo, useCallback, useEffect, useRef } from "react";
import { useResizeObserver } from "usehooks-ts";
import { actorLogsQueryOptions } from "../../queries";
import { useActorDetailsSettings } from "./actor-details-settings";
import { ActorConsoleMessage } from "./console/actor-console-message";

export type LogsTypeFilter = "all" | "errors";

interface ActorLogsProps {
	actorId: string;
	projectNameId: string;
	environmentNameId: string;
	typeFilter?: LogsTypeFilter;
	filter?: string;
}

export const ActorLogs = memo(
	({
		actorId,
		projectNameId,
		environmentNameId,
		typeFilter,
		filter,
	}: ActorLogsProps) => {
		const [settings] = useActorDetailsSettings();
		const follow = useRef(true);
		const shouldFollow = () => settings.autoFollowLogs && follow.current;

		const initialScrolled = useRef(false);
		const viewport = useRef<HTMLDivElement>(null);
		const virtualizer = useRef<Virtualizer<HTMLDivElement, Element>>(null);
		// Detect if the container has resized (i.e, console was opened)
		useResizeObserver({
			ref: viewport,
			onResize: () => {
				if (shouldFollow()) {
					// https://github.com/TanStack/virtual/issues/537
					requestAnimationFrame(() => {
						virtualizer.current?.scrollToIndex(sorted.length, {
							align: "end",
						});
					});
				}
			},
		});
		const [
			{ data: logs, isSuccess: isStdOutFetched },
			{ data: errors, isSuccess: isStdErrFetched },
		] = useQueries({
			queries: [
				actorLogsQueryOptions({
					actorId,
					projectNameId,
					environmentNameId,
					stream: "std_out",
				}),
				actorLogsQueryOptions({
					actorId,
					projectNameId,
					environmentNameId,
					stream: "std_err",
				}),
			],
		});

		const stdOutput =
			typeFilter === "errors"
				? []
				: (logs?.lines.map((log, index) => ({
						variant: log.includes("level=WARN")
							? ("warn" as const)
							: ("log" as const),
						message: log,
						timestamp: logs.timestamps[index],
						id: logs.ids[index],
					})) ?? []);

		const errOutput =
			errors?.lines.map((log, index) => ({
				variant: "error" as const,
				message: log,
				timestamp: errors.timestamps[index],
				id: errors.ids[index],
			})) ?? [];

		const output = [...stdOutput, ...errOutput];

		// Search
		const filtered =
			filter && filter.trim() !== ""
				? output.filter((log) => log.message.includes(filter))
				: output;

		const sorted = filtered.toSorted(
			(a, b) =>
				new Date(a.timestamp).valueOf() -
				new Date(b.timestamp).valueOf(),
		);

		const decorated = [
			...sorted,
			...(sorted.length === 0
				? ([
						{
							message:
								"No logs found. Logs are retained for 3 days.",
							variant: "warn",
							timestamp: "",
						},
					] as const)
				: []),
		];

		// Scroll to the bottom when new logs are added
		// biome-ignore lint/correctness/useExhaustiveDependencies: run this effect only when the length of the logs changes
		useEffect(() => {
			if (!shouldFollow()) {
				return () => {};
			}
			// https://github.com/TanStack/virtual/issues/537
			const rafId = requestAnimationFrame(() => {
				virtualizer.current?.scrollToIndex(
					virtualizer.current.options.count - 1,
					{
						align: "end",
					},
				);
			});

			return () => {
				cancelAnimationFrame(rafId);
			};
		}, [sorted.length]);

		// Scroll on mount
		useEffect(() => {
			if (
				isStdErrFetched &&
				isStdOutFetched &&
				!initialScrolled.current
			) {
				// https://github.com/TanStack/virtual/issues/537
				requestAnimationFrame(() => {
					virtualizer.current?.scrollToIndex(
						virtualizer.current.options.count - 1,
						{
							align: "end",
						},
					);
				});
				initialScrolled.current = true;
			}
		}, [isStdErrFetched, isStdOutFetched]);

		// Detect if the user has scrolled all the way to the bottom
		const handleChange = useCallback(
			(instance: Virtualizer<HTMLDivElement, Element>, sync: boolean) => {
				if (sync) {
					return;
				}

				follow.current =
					!instance.isScrolling &&
					instance.range?.endIndex === instance.options.count - 1;
			},
			[],
		);

		return (
			<VirtualScrollArea
				viewportRef={viewport}
				virtualizerRef={virtualizer}
				className="w-full flex-1 min-h-0"
				getRowData={(index) => ({
					...decorated[index],
					children: decorated[index].message,
					timestamp: settings.showTimestmaps
						? decorated[index].timestamp
						: undefined,
				})}
				onChange={handleChange}
				count={decorated.length}
				estimateSize={() => 26}
				row={ActorConsoleMessage}
			/>
		);
	},
);
