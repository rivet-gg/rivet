export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

export interface AgentState {
  messages: Message[];
  userMemory: Record<string, any>;
  toolData: Record<string, any>;
}

export interface WeatherData {
  temperature: number;
  feelsLike: number;
  humidity: number;
  windSpeed: number;
  windGust: number;
  conditions: string;
  location: string;
  timestamp: string;
  fallback?: boolean;
}

export interface ChatResponse {
  response: string;
  messageId: string;
  timestamp: number;
  toolCalls?: number;
}