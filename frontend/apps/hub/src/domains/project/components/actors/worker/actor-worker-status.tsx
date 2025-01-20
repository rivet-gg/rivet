import { Icon, faExclamationTriangle, faSpinner } from "@rivet-gg/icons";
import { AnimatePresence, motion } from "framer-motion";
import { useActorWorkerStatus } from "./actor-worker-context";

export function ActorWorkerStatus() {
	const status = useActorWorkerStatus();
	return (
		<AnimatePresence>
			{status.type === "pending" ? (
				<motion.span
					className="text-muted-foreground"
					exit={{ opacity: 0 }}
				>
					<Icon icon={faSpinner} className="animate-spin ml-2 mr-1" />
					Connecting to Actor...
				</motion.span>
			) : null}
			{status.type === "error" ? (
				<motion.span
					className="text-muted-destructive"
					exit={{ opacity: 0 }}
				>
					<Icon icon={faExclamationTriangle} className="ml-2 mr-1" />
					Couldn't connect to Actor, please try again.
				</motion.span>
			) : null}
		</AnimatePresence>
	);
}
