import {
	Button,
	RelativeTime,
	SmallText,
	WithTooltip,
	cn,
} from "@rivet-gg/components";
import { Icon, faTag, faTags } from "@rivet-gg/icons";
import { Link } from "@tanstack/react-router";
import { memo } from "react";
import { ActorRegion } from "./actor-region";
import { ActorTags } from "./actor-tags";
import { type ActorId } from "./queries";
import { useQuery } from "@tanstack/react-query";
import { QueriedActorStatusLabel } from "./actor-status-label";
import { QueriedActorStatusIndicator } from "./actor-status-indicator";
import { useManagerQueries } from "./manager-queries-context";

interface ActorsListRowProps {
	className?: string;
	actorId: ActorId;
	isCurrent?: boolean;
}

export const ActorsListRow = memo(
	({ className, actorId, isCurrent }: ActorsListRowProps) => {
		return (
			<Button
				className={cn(
					"h-auto grid grid-cols-subgrid col-span-full py-4 px-0 group border-l-0 border-r-0 border-t first-of-type:border-t-transparent border-b-transparent last-of-type:border-b-border rounded-none pr-4",
					className,
				)}
				variant={isCurrent ? "secondary" : "outline"}
				asChild
			>
				<Link
					to="."
					search={(search: Record<string, unknown>) => ({
						...search,
						actorId,
					})}
					className="min-w-0 flex-wrap gap-2"
				>
					<WithTooltip
						trigger={
							<div className="w-full flex justify-center">
								<QueriedActorStatusIndicator actorId={actorId} />
							</div>
						}
						content={<QueriedActorStatusLabel actorId={actorId} />}
					/>
					<Region actorId={actorId} />
					<Id actorId={actorId} />
					<Tags actorId={actorId} />

					<CreatedAt actorId={actorId} />
					<DestroyedAt actorId={actorId} />
				</Link>
			</Button>
		);
	},
);

function Id({ actorId }: { actorId: ActorId }) {
	return (
		<SmallText>
			{actorId.includes("-") ? actorId.split("-")[0] : actorId.substring(0, 8)}
		</SmallText>
	);
}

function Region({ actorId }: { actorId: ActorId }) {
	const { data: regionId } = useQuery(
		useManagerQueries().actorRegionQueryOptions(actorId),
	);

	if (!regionId) {
		return <SmallText className="text-muted-foreground">-</SmallText>;
	}

	return (
		<ActorRegion
			regionId={regionId}
			showLabel="abbreviated"
			className="[&_[data-slot=label]]:hidden @[500px]/table:[&_[data-slot=label]]:flex"
		/>
	);
}

function Tags({ actorId }: { actorId: ActorId }) {
	const { data: tags = {} } = useQuery(
		useManagerQueries().actorTagsQueryOptions(actorId),
	);

	const tagCount = Object.keys(tags).length;

	return (
		<WithTooltip
			trigger={
				<div className="relative overflow-r-gradient @container">
					<ActorTags
						className="flex-nowrap empty:block overflow-hidden @[150px]:block space-x-2 hidden"
						truncate={true}
						copy={false}
						tags={tags}
						hoverable={false}
						excludeBuiltIn="actors"
					/>
					<div className="block @[150px]:hidden text-xs text-muted-foreground">
						<Icon icon={tagCount === 1 ? faTag : faTags} className="mr-1" />
						{Object.keys(tags).length} {tagCount === 1 ? "tag" : "tags"}
					</div>
				</div>
			}
			content={
				<>
					<p className="pb-2 font-bold text-xs">Tags</p>
					<ActorTags
						excludeBuiltIn="actors"
						className="empty:block"
						copy={false}
						truncate={false}
						hoverable={false}
						tags={tags}
					/>
				</>
			}
		/>
	);
}

function CreatedAt({ actorId }: { actorId: ActorId }) {
	const { data: createdAt } = useQuery(
		useManagerQueries().actorCreatedAtQueryOptions(actorId),
	);

	return (
		<SmallText className="mx-1">
			{createdAt ? (
				<WithTooltip
					trigger={<RelativeTime time={createdAt} />}
					content={createdAt.toLocaleString()}
				/>
			) : (
				<span>-</span>
			)}
		</SmallText>
	);
}

function DestroyedAt({ actorId }: { actorId: ActorId }) {
	const { data: destroyedAt } = useQuery(
		useManagerQueries().actorDestroyedAtQueryOptions(actorId),
	);

	return (
		<SmallText className="mx-1">
			{destroyedAt ? (
				<WithTooltip
					trigger={<RelativeTime time={new Date(destroyedAt)} />}
					content={new Date(destroyedAt).toLocaleString()}
				/>
			) : (
				<span>-</span>
			)}
		</SmallText>
	);
}
