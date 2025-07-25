export function logServerStart(port: number, hasApiKey: boolean) {
  console.log("Starting Rivet + Mastra server on port", port);
  console.log("AI Model: OpenAI GPT-4o-mini");
  console.log("Tools: Weather API, Memory");
  console.log("Persistence: Rivet Actor state");
  console.log("Visit http://localhost:8080");
  console.log("OpenAI API Key:", hasApiKey ? "Loaded" : "Missing");
}

export function logActorCreated(actorName: string) {
  console.log(`Actor created: ${actorName}`);
}

export function logMastraAgentStart(actorName: string) {
  console.log(`Agent running: ${actorName}`);
}

export function logMastraAgentComplete(actorName: string, hasText: boolean) {
  console.log(`Agent completed: ${actorName}`);
}

export function logToolExecution(toolName: string, actorName: string) {
  console.log(`${toolName} tool executing: ${actorName}`);
}

export function logWeatherData(weatherData: any) {
  console.log(`Weather data saved:`, weatherData);
}

export function logActorCleared(actorName: string) {
  console.log(`Actor cleared: ${actorName}`);
}