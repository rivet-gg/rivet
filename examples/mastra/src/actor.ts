import { actor } from "@rivetkit/actor";
import dotenv from 'dotenv';
import { createChatAgent } from "./agents/chat-agent.js";
import { logActorCreated, logMastraAgentStart, logMastraAgentComplete, logActorCleared } from "./utils/logging.js";
import type { Message, AgentState, ChatResponse } from "./types/index.js";

// Load environment variables
dotenv.config();

// Rivet Actor containing a Pure Mastra Agent with real APIs
export const aiAgent = actor({
  state: <AgentState>{
    messages: [],
    userMemory: {},
    toolData: {}
  },

  onCreate: (c) => {
    c.state.userMemory.createdAt = Date.now();
    logActorCreated(c.name);
  },

  actions: {
    chat: async (c, message: string): Promise<ChatResponse> => {
      if (!process.env.OPENAI_API_KEY) {
        return { 
          response: "OpenAI API key not configured. Please set OPENAI_API_KEY in your .env file",
          messageId: crypto.randomUUID(),
          timestamp: Date.now()
        };
      }

      const userMessage: Message = {
        id: crypto.randomUUID(),
        role: "user",
        content: message,
        timestamp: Date.now()
      };
      c.state.messages.push(userMessage);
      
      // Keep conversation history manageable
      if (c.state.messages.length > 50) {
        c.state.messages = c.state.messages.slice(-50);
      }

      try {
        logMastraAgentStart(c.name);

        // Create Mastra agent with tools that access actor state
        const mastra = createChatAgent(c.state, c.name);
        const agent = await mastra.getAgent("ChatAgent");
        const result = await agent.generate(message);

        logMastraAgentComplete(c.name, !!result.text);

        const assistantMessage: Message = {
          id: crypto.randomUUID(),
          role: "assistant",
          content: result.text,
          timestamp: Date.now()
        };
        c.state.messages.push(assistantMessage);

        return { 
          response: result.text, 
          messageId: assistantMessage.id, 
          timestamp: assistantMessage.timestamp,
          toolCalls: result.toolCalls?.length || 0
        };

      } catch (error) {
        console.error('Mastra Agent error inside Rivet Actor:', error);
        
        let errorMessage = `Mastra Agent error: ${error.message}`;
        if (error.message && error.message.includes('quota')) {
          errorMessage = "OpenAI quota exceeded. Please check your API credits.";
        }
        
        const errorResponse: Message = {
          id: crypto.randomUUID(),
          role: "assistant", 
          content: errorMessage,
          timestamp: Date.now()
        };
        c.state.messages.push(errorResponse);

        return { 
          response: errorResponse.content, 
          messageId: errorResponse.id, 
          timestamp: errorResponse.timestamp
        };
      }
    },

    getHistory: (c, limit?: number) => {
      const messages = limit ? c.state.messages.slice(-limit) : c.state.messages;
      return { 
        history: messages, 
        total: c.state.messages.length,
        actorName: c.name,
        hasMemory: Object.keys(c.state.userMemory).length > 1,
        toolData: Object.keys(c.state.toolData)
      };
    },

    clear: (c) => {
      c.state.messages = [];
      c.state.userMemory = { createdAt: Date.now() };
      c.state.toolData = {};
      logActorCleared(c.name);
      return { success: true, message: "Rivet Actor + Mastra Agent cleared." };
    },

    getMetadata: (c) => ({
      userMemory: c.state.userMemory,
      toolData: c.state.toolData,
      actorName: c.name,
      messageCount: c.state.messages.length
    })
  }
});