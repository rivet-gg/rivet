import { Button } from "@rivet-gg/components";
import { type CodeMirrorRef, JsonCode } from "@rivet-gg/components/code-mirror";
import { AnimatePresence, motion } from "framer-motion";
import { useMemo, useRef, useState } from "react";
import type { ContainerState } from "./worker/actor-worker-container";
import { useActorWorker } from "./worker/actor-worker-context";

const isValidJson = (json: string) => {
	try {
		JSON.parse(json);
		return true;
	} catch {
		return false;
	}
};

interface ActorEditableStateProps {
	state: ContainerState["state"];
}

export function ActorEditableState({ state }: ActorEditableStateProps) {
	const container = useActorWorker();
	const [isEditing, setIsEditing] = useState(false);
	const [value, setValue] = useState<string | null>(null);

	const ref = useRef<CodeMirrorRef>(null);

	const formatted = useMemo(() => {
		return JSON.stringify(JSON.parse(state.native || ""), null, 2);
	}, [state.native]);

	return (
		<>
			<JsonCode
				ref={ref}
				value={value || formatted}
				onChange={(value) => {
					if (!isEditing) {
						setValue(formatted);
					} else {
						setValue(value);
					}

					setIsEditing(true);
				}}
			/>
			<AnimatePresence>
				{isEditing ? (
					<motion.div
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						className="mt-1 flex gap-2 justify-end"
					>
						<Button
							variant="ghost"
							onClick={() => {
								setValue(null);
								setIsEditing(false);
							}}
						>
							Reset
						</Button>
						<Button
							disabled={!isValidJson(value || "")}
							onClick={() => {
								container.setState(value || "");
								setIsEditing(false);
								setValue(null);
							}}
						>
							Save
						</Button>
					</motion.div>
				) : null}
			</AnimatePresence>
		</>
	);
}
