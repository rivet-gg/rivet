import { Button } from "@rivet-gg/components";
import { Icon, faChevronDown } from "@rivet-gg/icons";
import { AnimatePresence, motion } from "framer-motion";
import { useState } from "react";
import { ActorWorkerStatus } from "../worker/actor-worker-status";
import { ActorConsoleInput } from "./actor-console-input";
import { ActorConsoleLogs } from "./actor-console-logs";

export function ActorConsole() {
	const [isOpen, setOpen] = useState(false);

	return (
		<motion.div
			animate={{
				height: isOpen ? "50%" : "36px",
			}}
			className="overflow-hidden flex flex-col"
		>
			<Button
				variant="ghost"
				onClick={() => setOpen((old) => !old)}
				className="border-t border-b rounded-none w-full justify-between min-h-9"
				size="sm"
				endIcon={<Icon icon={faChevronDown} />}
			>
				<span>
					Console
					<ActorWorkerStatus />
				</span>
			</Button>
			<AnimatePresence>
				{isOpen ? (
					<motion.div
						exit={{ opacity: 0 }}
						className="flex flex-col flex-1 max-h-full overflow-hidden"
					>
						<ActorConsoleLogs />
						<ActorConsoleInput />
					</motion.div>
				) : null}
			</AnimatePresence>
		</motion.div>
	);
}
