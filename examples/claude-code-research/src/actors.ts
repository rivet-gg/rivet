import { claudeCodeAgent, runClaudeCode } from "@rivet-dev/claude-code";
import { actor, event, setup, UserError } from "rivetkit";
import { z } from "zod";

const PLANNER_SYSTEM_PROMPT =
	"You plan research. Split the question you are given into 2 to 4 independent sub-questions that together answer it.";
const RESEARCHER_SYSTEM_PROMPT =
	"You are a research agent. Investigate the question you are given with web search and web fetch. Reply with a concise, factual summary of what you found and cite the source URLs.";
const SYNTHESIZER_SYSTEM_PROMPT =
	"You write research reports. Combine the findings you are given into a clear answer to the original question and keep the source URLs.";

const researchPlanSchema = z.object({
	subQuestions: z.array(z.string()).min(2).max(4),
});

export type ResearchStatus =
	| "idle"
	| "planning"
	| "investigating"
	| "synthesizing"
	| "done"
	| "failed";

export interface ResearchProgress {
	status: ResearchStatus;
	message: string;
}

export interface Finding {
	subQuestion: string;
	answer: string;
	costUsd: number;
}

export const researcher = claudeCodeAgent({
	systemPrompt: RESEARCHER_SYSTEM_PROMPT,
	tools: ["WebSearch", "WebFetch"],
	model: process.env.RESEARCH_MODEL,
});

export const researchCoordinator = actor({
	state: {
		question: null as string | null,
		status: "idle" as ResearchStatus,
		findings: [] as Finding[],
		report: null as string | null,
		error: null as string | null,
		costUsd: 0,
	},
	events: {
		progress: event<ResearchProgress>(),
	},
	actions: {
		start: (c, question: string) => {
			if (c.state.status !== "idle") {
				throw new UserError("This research has already started.");
			}

			const model = process.env.RESEARCH_MODEL;
			const researchers = c.client<typeof registry>().researcher;

			const reportProgress = (
				status: ResearchStatus,
				message: string,
			) => {
				c.state.status = status;
				c.broadcast("progress", { status, message });
			};

			const investigate = async (subQuestion: string, index: number) => {
				const turn = await researchers
					.getOrCreate([...c.key, `${index}`])
					.prompt(subQuestion);
				c.state.findings.push({
					subQuestion,
					answer: turn.response,
					costUsd: turn.costUsd,
				});
				c.state.costUsd += turn.costUsd;
				reportProgress("investigating", `Finished: ${subQuestion}`);
			};

			const research = async () => {
				try {
					reportProgress("planning", "Planning sub-questions");
					const plan = await runClaudeCode(
						{
							prompt: question,
							systemPrompt: PLANNER_SYSTEM_PROMPT,
							model,
						},
						researchPlanSchema,
					);
					c.state.costUsd += plan.costUsd;

					const { subQuestions } = plan.output;
					reportProgress(
						"investigating",
						`Investigating ${subQuestions.length} sub-questions in parallel`,
					);
					await Promise.all(subQuestions.map(investigate));

					reportProgress("synthesizing", "Writing the report");
					const report = await runClaudeCode({
						prompt: buildSynthesisPrompt(
							question,
							c.state.findings,
						),
						systemPrompt: SYNTHESIZER_SYSTEM_PROMPT,
						model,
					});
					c.state.report = report.output;
					c.state.costUsd += report.costUsd;
					reportProgress("done", "Report ready");
				} catch (error) {
					c.state.error =
						error instanceof Error ? error.message : String(error);
					reportProgress("failed", c.state.error);
				}
			};

			c.state.question = question;
			void c.keepAwake(research());
			return { status: c.state.status };
		},
		getResult: (c) => ({ ...c.state }),
	},
});

function buildSynthesisPrompt(question: string, findings: Finding[]): string {
	const findingSections = findings.map(
		(finding) => `## ${finding.subQuestion}\n\n${finding.answer}`,
	);
	return [`# Question\n\n${question}`, "# Findings", ...findingSections].join(
		"\n\n",
	);
}

export const registry = setup({
	use: { researchCoordinator, researcher },
});
