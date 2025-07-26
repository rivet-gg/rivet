import { Agent, Mastra } from "@mastra/core";
import { openai } from "@ai-sdk/openai";
import { createWeatherTool, createMemoryTool, createRecallTool } from "../tools/index.js";
import type { AgentState } from "../types/index.js";

const AGENT_INSTRUCTIONS = `You are an AI assistant with persistent memory and tools.

Available tools:
- get-weather: Get current weather for any location
- remember: Save information to memory
- recall: Retrieve saved information

Your conversations and data persist between sessions. Use tools when helpful and provide clear, concise responses.`;

export function createChatAgent(state: AgentState, actorName: string) {
  const toolContext = { state, actorName };
  
  const chatAgent = new Agent({
    name: "ChatAgent",
    instructions: AGENT_INSTRUCTIONS,
    model: openai('gpt-4o-mini'),
    tools: { 
      weatherTool: createWeatherTool(toolContext),
      memoryTool: createMemoryTool(toolContext),
      recallTool: createRecallTool(toolContext)
    }
  });

  const mastra = new Mastra({
    agents: { ChatAgent: chatAgent }
  });

  return mastra;
}