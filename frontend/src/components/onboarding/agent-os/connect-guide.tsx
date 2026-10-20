import { faCode, faTypescript, Icon } from "@rivet-gg/icons";
import { AgentPromptBanner, CommandBox } from "@/app/compute-deploy";
import { CodeFrame } from "@/components/code";
import { CodePreview } from "@/components/code-preview/code-preview";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { getOnboardingTargetCopy } from "@/content/agent-prompts";
import {
	AGENT_OS_INSTALL_COMMAND,
	getAgentOsCurlExample,
	getAgentOsPrompt,
	getAgentOsTypeScriptExample,
} from "./connect-examples";

export type AgentOsExample = "typescript" | "http";

export function AgentOsConnectGuide({
	privateEndpoint,
	namespace,
	token,
	example,
	onExampleChange,
}: {
	privateEndpoint: string;
	namespace: string;
	token: string;
	example: AgentOsExample;
	onExampleChange: (example: AgentOsExample) => void;
}) {
	const apiUrl = new URL(privateEndpoint);
	apiUrl.username = "";
	apiUrl.password = "";
	const config = {
		apiUrl: apiUrl.toString(),
		namespace,
		token,
	};
	const typescriptCode = getAgentOsTypeScriptExample(config);
	const curlCode = getAgentOsCurlExample(config);
	const code = example === "typescript" ? typescriptCode : curlCode;
	const hasToken = token.length > 0;

	return (
		<div className="flex flex-col gap-6">
			<AgentPromptBanner
				code={getAgentOsPrompt(example, code, {
					includesToken: hasToken,
				})}
				containsSecret={hasToken}
				secretName="private token"
				description={`Have your coding agent connect using the ${example === "typescript" ? "TypeScript" : "curl"} example below.`}
			/>
			<div className="flex items-center gap-3">
				<div className="h-px flex-1 bg-border" />
				<span className="text-xs text-muted-foreground">
					or set it up yourself
				</span>
				<div className="h-px flex-1 bg-border" />
			</div>
			<p className="text-sm text-muted-foreground">
				{hasToken
					? "agentOS runs as a managed service. This example includes your private token, so use it from server-side code and keep it out of source control."
					: "agentOS runs as a managed service. Run this example from server-side code."}
			</p>
			<Tabs
				className="rounded-lg border"
				value={example}
				onValueChange={(value) =>
					onExampleChange(value as AgentOsExample)
				}
			>
				<TabsList className="pt-2">
					<TabsTrigger value="typescript">
						<Icon icon={faTypescript} className="mr-1.5" />
						TypeScript
					</TabsTrigger>
					<TabsTrigger value="http">
						<Icon icon={faCode} className="mr-1.5" />
						curl
					</TabsTrigger>
				</TabsList>
				<TabsContent value="typescript" className="m-0 space-y-4 p-4">
					<div>
						<p className="mb-1.5 text-sm font-medium">Install</p>
						<CommandBox command={AGENT_OS_INSTALL_COMMAND} />
					</div>
					<AgentOsCopyableCode
						language="typescript"
						code={typescriptCode}
						token={token}
					/>
				</TabsContent>
				<TabsContent value="http" className="m-0 p-4">
					<AgentOsCopyableCode
						language="bash"
						code={curlCode}
						token={token}
					/>
				</TabsContent>
			</Tabs>
			<a
				href={getOnboardingTargetCopy("agent-os").quickstartUrl}
				target="_blank"
				rel="noopener noreferrer"
				className="text-sm text-primary hover:underline"
			>
				Read the agentOS docs →
			</a>
		</div>
	);
}

function AgentOsCopyableCode({
	language,
	code,
	token,
}: {
	language: "typescript" | "bash";
	code: string;
	token: string;
}) {
	const secret =
		language === "typescript"
			? JSON.stringify(token).slice(1, -1)
			: token.replaceAll("'", `'"'"'`);
	const secretStart = code.indexOf(secret);
	return (
		<div>
			<CodeFrame
				language={language}
				code={() => code}
				className="m-0"
				hideFooter
			>
				<CodePreview
					language={language}
					className="text-left"
					code={code}
					decorations={
						secret && secretStart !== -1
							? [
									{
										start: secretStart,
										end: secretStart + secret.length,
										alwaysWrap: true,
										properties: {
											class: "relative after:absolute after:inset-0 after:rounded-sm after:bg-muted-foreground after:content-[''] hover:after:opacity-0 focus:after:opacity-0",
											tabIndex: 0,
											"aria-label":
												"Private token. Hover or focus to reveal.",
										},
									},
								]
							: undefined
					}
				/>
			</CodeFrame>
			{token ? (
				<p className="mt-2 text-xs text-muted-foreground">
					Hover to reveal the private token. Copy includes the full
					token.
				</p>
			) : null}
		</div>
	);
}
