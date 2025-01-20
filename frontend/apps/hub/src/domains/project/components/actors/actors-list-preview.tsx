import { ls } from "@/lib/ls";
import { Button, WithTooltip, cn } from "@rivet-gg/components";
import {
	Icon,
	faGripDotsVertical,
	faLeftFromLine,
	faRightFromLine,
} from "@rivet-gg/icons";
import {
	animate,
	motion,
	useMotionTemplate,
	useMotionValue,
	useMotionValueEvent,
} from "framer-motion";
import { Suspense, memo, useLayoutEffect, useState } from "react";
import { ActorsActorDetailsPanel } from "./actors-actor-details-panel";
import { ActorsListPanel } from "./actors-list-panel";

const RIGHT_PANEL_MIN_WIDTH = 460;

interface ActorsListPreview {
	projectNameId: string;
	environmentNameId: string;
	actorId?: string;
}

export const ActorsListPreview = memo(
	({ projectNameId, environmentNameId, actorId }: ActorsListPreview) => {
		const outerWidth = useMotionValue(0);
		const x = useMotionValue(0);

		const rightWidth = useMotionTemplate`calc(50% - ${x}px)`;
		const leftWidth = useMotionTemplate`calc(50% + ${x}px)`;

		const [folded, setFolded] = useState(() => ls.actorsList.getFolded());
		const [isDragging, setIsDragging] = useState(false);

		const [, setInitialized] = useState(false);

		useMotionValueEvent(x, "change", (value) => {
			ls.actorsList.set(value / outerWidth.get(), folded);
		});

		// biome-ignore lint/correctness/useExhaustiveDependencies: on first draw
		useLayoutEffect(() => {
			x.setCurrent((ls.actorsList.getWidth() || 0) * outerWidth.get());
			setInitialized(true);
		}, []);
		return (
			<div
				className="w-full relative flex flex-row h-full flex-1 max-h-full min-h-0 overflow-hidden"
				ref={(ref) => {
					if (ref) {
						const width = ref.getBoundingClientRect().width;
						outerWidth.set(width);
					}
				}}
			>
				<motion.div
					className={cn(
						"py-2 px-1 bg-card relative transition-colors border-r-border border-r",
						folded && "border-r-transparent",
					)}
				>
					<WithTooltip
						trigger={
							<Button
								variant="ghost"
								size="icon"
								onClick={() => {
									setFolded(!folded);
									if (folded) {
										animate(x, 0);
									} else {
										animate(x, -outerWidth.get() / 2);
									}
								}}
							>
								<Icon
									icon={
										folded
											? faRightFromLine
											: faLeftFromLine
									}
								/>
							</Button>
						}
						content={folded ? "Expand" : "Collapse"}
					/>
				</motion.div>
				<motion.div
					className="h-full min-h-0 flex overflow-hidden"
					style={{
						width: leftWidth,
					}}
					animate={{
						opacity: folded ? 0 : 1,
					}}
				>
					<ActorsListPanel
						projectNameId={projectNameId}
						environmentNameId={environmentNameId}
						actorId={actorId}
					/>
				</motion.div>
				<motion.div
					drag="x"
					_dragX={x}
					onDragStart={() => setIsDragging(true)}
					onDrag={(e, info) => {
						const rightPos = outerWidth.get() - info.point.x;
						setFolded(outerWidth.get() - rightPos < 470);
					}}
					onDragEnd={(e, info) => {
						if (folded) {
							animate(x, -outerWidth.get() / 2);
						} else {
							const leftPos = outerWidth.get() - info.point.x;
							if (leftPos < RIGHT_PANEL_MIN_WIDTH) {
								animate(
									x,
									outerWidth.get() / 2 -
										RIGHT_PANEL_MIN_WIDTH,
								);
							}
						}
						setIsDragging(false);
					}}
					dragMomentum={false}
					className="w-[1px] bg-border cursor-col-resize inset-y-0 z-20 relative flex items-center group"
				>
					<Icon
						icon={faGripDotsVertical}
						className={cn(
							"w-4 h-4 -translate-x-1/2 group-hover:opacity-100 opacity-10 transition-all scale-100",
							isDragging && "opacity-100 scale-125",
						)}
					/>
				</motion.div>
				<motion.div
					className="h-full max-h-full overflow-hidden flex flex-col flex-grow"
					style={{
						width: rightWidth,
						minWidth: RIGHT_PANEL_MIN_WIDTH,
					}}
				>
					<Suspense fallback={<ActorsActorDetailsPanel.Skeleton />}>
						<ActorsActorDetailsPanel
							projectNameId={projectNameId}
							environmentNameId={environmentNameId}
							actorId={actorId}
						/>
					</Suspense>
				</motion.div>
			</div>
		);
	},
);
