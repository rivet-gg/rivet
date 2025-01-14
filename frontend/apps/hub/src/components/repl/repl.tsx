import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Code, WithTooltip } from "@rivet-gg/components";
import { faChevronRight } from "@rivet-gg/icons";
import { Fragment } from "react/jsx-runtime";
import { ReplInput } from "./repl-input";
import { ReplLog } from "./repl-log";
import { useRepl } from "./repl-state";

interface ReplProps {
	rpcs: string[];
	actorId: string;
	managerUrl: string;
}

export function ActorRepl({ rpcs, actorId, managerUrl }: ReplProps) {
	const [commands, runCode] = useRepl();
	return (
		<div className="border p-4 rounded-md text-xs">
			<ReplLog commands={commands} />
			<div className="flex items-start gap-2 mt-2">
				<div className="min-w-4 text-center">
					<FontAwesomeIcon icon={faChevronRight} />
				</div>
				<div className="flex-1">
					<ReplInput
						rpcs={rpcs}
						onRun={(code) => {
							runCode({ code, actorId, managerUrl, rpcs });
						}}
					/>
				</div>
			</div>
			<p className="text-muted-foreground mt-1 pl-1">
				You can call all defined RPCs here:{" "}
				{rpcs.map((rpc, index) => (
					<Fragment
						// biome-ignore lint/suspicious/noArrayIndexKey: we're using the index as a key here because the array is static
						key={index}
					>
						<Code className="text-xs">{rpc}</Code>
						{index < rpcs.length - 1 ? ", " : ""}
					</Fragment>
				))}
				. Available helpers:
				<WithTooltip
					trigger={<Code className="text-xs">wait</Code>}
					content="Helper function to wait for a number of milliseconds"
				/>
				.
				<br />
				Press Shift+Enter to run your command.
			</p>
		</div>
	);
}
