export const AGENT_OS_INSTALL_COMMAND = "npm install @rivet-dev/agentos";

export function getAgentOsTypeScriptExample({
	apiUrl,
	namespace,
	token,
}: {
	apiUrl: string;
	namespace: string;
	token: string;
}) {
	const tokenLine = token ? `\n\ttoken: ${JSON.stringify(token)},` : "";
	return `import { createAgentOsClient } from "@rivet-dev/agentos";

const client = createAgentOsClient({
	endpoint: ${JSON.stringify(apiUrl)},
	namespace: ${JSON.stringify(namespace)},${tokenLine}
});

const vm = client.agentOS.getOrCreate(["my-vm"]);

const result = await vm.v1.process.run({
	command: "sh",
	args: ["-c", "echo hello"],
	options: { captureStdio: true },
});

console.log(result.stdout);`;
}

function shellQuote(value: string) {
	return `'${value.replaceAll("'", `'"'"'`)}'`;
}

export function getAgentOsCurlExample({
	apiUrl,
	namespace,
	token,
}: {
	apiUrl: string;
	namespace: string;
	token: string;
}) {
	const url = new URL(apiUrl);
	const basePath = url.pathname.replace(/\/$/, "");
	const query = new URLSearchParams({
		"rvt-namespace": namespace,
		"rvt-method": "getOrCreate",
		"rvt-runner": "services",
		"rvt-key": "my-vm",
		"rvt-crash-policy": "sleep",
	});
	const action = JSON.stringify({
		args: [
			{
				command: "sh",
				args: ["-c", "echo hello"],
				options: { captureStdio: true },
			},
		],
	});
	const actionUrl = `${url.origin}${basePath}/gateway/agentOS/action/v1.process.run?${query}`;
	const tokenHeader = token
		? `\n  -H ${shellQuote(`x-rivet-token: ${token}`)} \\`
		: "";
	return `curl -X POST ${shellQuote(actionUrl)} \\${tokenHeader}
  -H 'x-rivet-encoding: json' \\
  -H 'Content-Type: application/json' \\
  --data ${shellQuote(action)}`;
}

export function getAgentOsPrompt(
	example: "typescript" | "http",
	code: string,
	{ includesToken = true }: { includesToken?: boolean } = {},
) {
	const tokenNote = includesToken
		? " The example includes a private Rivet token; keep it out of browser code and source control."
		: "";
	return `Connect to the managed agentOS service from server-side code.${tokenNote}

${example === "typescript" ? `${AGENT_OS_INSTALL_COMMAND}\n\n` : ""}${code}

Read the agentOS docs for more: https://rivet.dev/agentos/docs/`;
}
