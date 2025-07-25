import { createTool } from "@mastra/core";
import { z } from "zod";
import { getWeatherData } from "../utils/weather.js";
import { logToolExecution, logWeatherData } from "../utils/logging.js";
import type { AgentState } from "../types/index.js";

interface ToolContext {
  state: AgentState;
  actorName: string;
}

export function createWeatherTool(context: ToolContext) {
  return createTool({
    id: "get-weather",
    description: "Get current weather for a location",
    inputSchema: z.object({
      location: z.string().describe("City name")
    }),
    outputSchema: z.object({
      temperature: z.number(),
      feelsLike: z.number(),
      humidity: z.number(),
      windSpeed: z.number(),
      windGust: z.number(),
      conditions: z.string(),
      location: z.string()
    }),
    execute: async ({ context: toolContext }) => {
      logToolExecution("weather", context.actorName);
      
      const weatherData = await getWeatherData(toolContext.location);
      
      // Save to Rivet Actor state
      if (!context.state.toolData) context.state.toolData = {};
      context.state.toolData.lastWeather = weatherData;
      
      logWeatherData(weatherData);
      return weatherData;
    }
  });
}


export function createMemoryTool(context: ToolContext) {
  return createTool({
    id: "remember",
    description: "Save information to persistent memory",
    inputSchema: z.object({
      information: z.string().describe("Information to remember")
    }),
    outputSchema: z.object({
      information: z.string(),
      saved: z.boolean(),
      message: z.string()
    }),
    execute: async ({ context: toolContext }) => {
      logToolExecution("memory", context.actorName);
      
      if (!context.state.userMemory) context.state.userMemory = {};
      context.state.userMemory.content = toolContext.information;
      context.state.userMemory.savedAt = new Date().toISOString();
      
      return {
        information: toolContext.information,
        saved: true,
        message: `I've remembered: "${toolContext.information}"`
      };
    }
  });
}

export function createRecallTool(context: ToolContext) {
  return createTool({
    id: "recall",
    description: "Retrieve information from persistent memory",
    inputSchema: z.object({
      query: z.string().optional().describe("What to recall")
    }),
    outputSchema: z.object({
      memories: z.array(z.string()),
      count: z.number(),
      message: z.string()
    }),
    execute: async ({ context: toolContext }) => {
      logToolExecution("recall", context.actorName);
      
      if (!context.state.userMemory) context.state.userMemory = {};
      if (!context.state.toolData) context.state.toolData = {};
      
      let memories = [];
      
      if (context.state.userMemory.content) {
        memories.push(`You told me: "${context.state.userMemory.content}"`);
      }
      
      if (context.state.toolData.lastWeather) {
        const w = context.state.toolData.lastWeather;
        memories.push(`Last weather: ${w.location} - ${w.conditions}, ${w.temperature}°C`);
      }
      
      
      const message = memories.length === 0 
        ? "I don't have any memories stored yet."
        : `Here's what I remember:\n\n${memories.join('\n')}`;
      
      return { memories, count: memories.length, message };
    }
  });
}