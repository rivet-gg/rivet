import type { Story } from "@ladle/react";
import { faArrowLeft, Icon } from "@rivet-gg/icons";
import { useState } from "react";
import "../../../../.ladle/ladle.css";
import { AgentPromptBanner } from "@/app/compute-deploy";
import { DurableStreamsManagedService } from "@/app/getting-started";
import { ProductPicker } from "@/components/products/product-picker";
import { Button } from "@/components/ui/button";
import { TooltipProvider } from "@/components/ui/tooltip";
import { getAgentInstructionsPrompt } from "@/content/agent-prompts";
import { AgentOsConnectGuide, type AgentOsExample } from "./connect-guide";

function PreviewCard({
	step,
	title,
	children,
	onBack,
}: {
	step: number;
	title: string;
	children: React.ReactNode;
	onBack?: () => void;
}) {
	return (
		<main className="dark min-h-screen bg-background text-foreground flex items-center justify-center p-8">
			<div className="w-full max-w-[36rem] rounded-xl border bg-card p-8 shadow-sm">
				<div className="mb-6 flex gap-1.5">
					<div className="h-1 flex-1 rounded-full bg-primary" />
					<div
						className={`h-1 flex-1 rounded-full ${step === 2 ? "bg-primary" : "bg-muted"}`}
					/>
				</div>
				<p className="mb-2 text-xs text-muted-foreground">
					Step {step} of 2
				</p>
				<h1 className="mb-6 text-xl font-semibold">{title}</h1>
				{children}
				{step === 2 ? (
					<div className="mt-6 flex justify-end gap-4">
						<Button
							type="button"
							variant="outline"
							onClick={onBack}
						>
							<Icon icon={faArrowLeft} className="mr-2" />
							Back
						</Button>
						<Button type="button">Done</Button>
					</div>
				) : null}
			</div>
		</main>
	);
}

export const SelectAgentOs: Story = () => {
	const [selected, setSelected] = useState(false);
	return selected ? (
		<Connect
			initialExample="typescript"
			onBack={() => setSelected(false)}
		/>
	) : (
		<PreviewCard step={1} title="Select a product">
			<ProductPicker
				onSelect={(target) => setSelected(target === "agent-os")}
			/>
		</PreviewCard>
	);
};

function Connect({
	initialExample,
	onBack,
	token = "private_example_token",
}: {
	initialExample: AgentOsExample;
	onBack?: () => void;
	token?: string;
}) {
	const [example, setExample] = useState(initialExample);
	return (
		<PreviewCard step={2} title="Connect to agentOS" onBack={onBack}>
			<AgentOsConnectGuide
				privateEndpoint={`https://demo:${token}@api.rivet.dev`}
				namespace="demo"
				token={token}
				example={example}
				onExampleChange={setExample}
			/>
		</PreviewCard>
	);
}

export const TypeScript: Story = () => <Connect initialExample="typescript" />;
export const Curl: Story = () => <Connect initialExample="http" />;
export const WithoutToken: Story = () => (
	<Connect initialExample="typescript" token="" />
);

export const DurableStreams: Story = () => {
	const serviceUrl = "https://services--demo.rivet.run/durable-streams/";
	return (
		<TooltipProvider>
			<PreviewCard step={2} title="Connect to Durable Streams">
				<div className="flex flex-col gap-6">
					<AgentPromptBanner
						code={getAgentInstructionsPrompt({
							providerStr: "Rivet Cloud",
							publishableToken: "",
							secretToken: "",
							runnerName: "services",
							serverless: false,
							namespace: "demo",
							target: "durable-streams",
							durableStreamsServiceUrl: serviceUrl,
						})}
						description="Have your coding agent point your client at the managed Durable Streams service."
					/>
					<div className="flex items-center gap-3">
						<div className="h-px flex-1 bg-border" />
						<span className="text-xs text-muted-foreground">
							or set it up manually
						</span>
						<div className="h-px flex-1 bg-border" />
					</div>
					<DurableStreamsManagedService serviceUrl={serviceUrl} />
				</div>
			</PreviewCard>
		</TooltipProvider>
	);
};
