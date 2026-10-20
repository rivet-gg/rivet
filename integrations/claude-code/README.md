# @rivet-dev/claude-code

Run [Claude Code](https://code.claude.com) agents as durable [Rivet](https://rivet.dev) Actors. Each agent runs the `claude` CLI under your own Claude Code login, so no API key is needed.

Each actor is one persistent Claude Code session. Prompts continue the same conversation, and every turn is stored in actor state and broadcast as a `turn` event.

Use your own login for your own local work. Do not use it to serve other people.

```sh
npm add @rivet-dev/claude-code
```

## Agents

```ts
import { claudeCodeAgent } from "@rivet-dev/claude-code";
import { setup } from "rivetkit";

const researcher = claudeCodeAgent({
	systemPrompt: "You research questions with web search and cite your sources.",
	tools: ["WebSearch", "WebFetch"],
});

export const registry = setup({ use: { researcher } });
registry.start();
```

```ts
const agent = client.researcher.getOrCreate(["rivet-vs-durable-objects"]);
const turn = await agent.prompt("How do Rivet Actors compare to Durable Objects?");
const followUp = await agent.prompt("Which one fits long-running coding agents?");
```

| Option | Description |
|---|---|
| `systemPrompt` | The agent's role. Replaces Claude Code's default system prompt. |
| `tools` | Claude Code tools the agent may use, approved automatically. No tools when omitted. |
| `model` | Model for every turn. Defaults to your Claude Code default. |
| `cwd` | Working directory, for example a repository a coding agent works in. |
| `turnTimeoutMs` | Maximum time for one turn. Defaults to 10 minutes. |

An agent works on one prompt at a time. A prompt sent while a turn is running fails with `claude_code_turn_in_progress`.

## One-off runs

`runClaudeCode` runs Claude Code once. Pass a Zod schema to get validated structured output.

```ts
import { runClaudeCode } from "@rivet-dev/claude-code";
import { z } from "zod";

const plan = await runClaudeCode(
	{ prompt: "Plan research on Rivet Actors", systemPrompt: "You plan research." },
	z.object({ subQuestions: z.array(z.string()) }),
);
```

## Requirements

- Claude Code installed and logged in. Run `claude` once.
- Set `CLAUDE_CODE_BIN` if `claude` is not on your `PATH`.
