import { cn } from "@rivet-gg/components";
import {
	Icon,
	faArrowLeft,
	faArrowRight,
	faCircleX,
	faSpinner,
} from "@rivet-gg/icons";
import type { ReplCommand } from "./repl-state";

interface ReplLogProps {
	commands: ReplCommand[];
}

export function ReplLog({ commands }: ReplLogProps) {
	return (
		<div className="flex flex-col gap-3">
			{commands.map((command) => (
				<div key={command.key} className="flex flex-col gap-1">
					<div className="text-xs font-mono text-muted-foreground flex gap-1">
						<div className="min-w-4 text-center">
							<Icon icon={faArrowRight} />
						</div>

						{command.status === "pending" ? (
							<div className="text-muted-foreground text-xs font-mono">
								...
							</div>
						) : null}

						{command.status !== "pending" && command.formatted ? (
							<div
								className="text-xs font-mono text-muted-foreground"
								style={{ color: command.formatted.fg }}
							>
								{command.formatted?.tokens.map(
									(tokensLine, index) => (
										<span
											// biome-ignore lint/suspicious/noArrayIndexKey: we're using the index as a key here because the array is static
											key={index}
											className="block"
										>
											{tokensLine.map((token, index) => (
												<span
													// biome-ignore lint/suspicious/noArrayIndexKey: we're using the index as a key here because the array is static
													key={index}
													style={{
														color: token.color,
													}}
													className="whitespace-pre"
												>
													{token.content}
												</span>
											))}
										</span>
									),
								)}
							</div>
						) : null}
					</div>
					<div className="flex flex-col gap-1 pl-4 max-w-full min-w-0">
						{command.logs?.map((log, index) => (
							<div
								// biome-ignore lint/suspicious/noArrayIndexKey: we're using the index as a key here because the array is static
								key={index}
								className={cn(
									"text-xs font-mono break-words min-w-0 max-w-100",
									{
										"text-muted-destructive":
											log.level === "error",
									},
								)}
							>
								{log.message}
							</div>
						))}
					</div>
					{command.status !== "success" &&
					command.status !== "error" ? (
						<div className="text-xs font-mono text-muted-foreground flex py-0.5 gap-1">
							<div className="min-w-4 text-center">
								<Icon
									icon={faSpinner}
									className="animate-spin"
								/>
							</div>
							Waiting for response...
						</div>
					) : null}
					{command.status === "success" ? (
						<div className="text-xs font-mono text-muted-foreground flex py-0.5 gap-1 w-full">
							<div className="min-w-4 text-center">
								<Icon icon={faArrowLeft} />
							</div>

							<div className="break-words min-w-0 max-w-full">
								{(command.result as string) || "undefined"}
							</div>
						</div>
					) : null}
					{command.status === "error" ? (
						<div className="text-xs font-mono bg-muted-destructive/50 flex py-0.5 gap-1">
							<div className="min-w-4 text-center">
								<Icon icon={faCircleX} />
							</div>

							<div className="break-words">
								{typeof command.error === "object" &&
								command.error &&
								"message" in command.error
									? (command.error.message as string) ||
										"undefined"
									: "undefined"}
							</div>
						</div>
					) : null}
				</div>
			))}
		</div>
	);
}
