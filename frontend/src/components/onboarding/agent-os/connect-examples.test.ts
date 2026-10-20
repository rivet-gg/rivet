import { describe, expect, it } from "vitest";
import {
	AGENT_OS_INSTALL_COMMAND,
	getAgentOsCurlExample,
	getAgentOsPrompt,
	getAgentOsTypeScriptExample,
} from "./connect-examples";

describe("managed agentOS connection examples", () => {
	it("passes the namespace and private token explicitly to the generated client", () => {
		const code = getAgentOsTypeScriptExample({
			apiUrl: "https://api.rivet.dev/",
			namespace: "demo",
			token: "private_test",
		});
		expect(code).toContain('endpoint: "https://api.rivet.dev/"');
		expect(code).toContain('namespace: "demo"');
		expect(code).toContain('token: "private_test"');
		expect(code).toContain('client.agentOS.getOrCreate(["my-vm"])');
		expect(code).toContain("vm.v1.process.run");
	});

	it("gets or creates the VM by key in one authenticated curl request", () => {
		const code = getAgentOsCurlExample({
			apiUrl: "https://api.rivet.dev/prefix/",
			namespace: "demo namespace",
			token: "private_test",
		});
		const actionUrl = code.match(/^curl -X POST '([^']+)'/)?.[1];
		expect(actionUrl).toBeDefined();
		const url = new URL(actionUrl!);
		expect(url.pathname).toBe(
			"/prefix/gateway/agentOS/action/v1.process.run",
		);
		expect(url.searchParams.get("rvt-namespace")).toBe("demo namespace");
		expect(url.searchParams.get("rvt-method")).toBe("getOrCreate");
		expect(url.searchParams.get("rvt-runner")).toBe("services");
		expect(url.searchParams.get("rvt-key")).toBe("my-vm");
		expect(url.searchParams.get("rvt-crash-policy")).toBe("sleep");
		expect(url.searchParams.has("rvt-token")).toBe(false);
		expect(code).toContain("-H 'x-rivet-token: private_test'");
		const body = code.match(/--data '([^']+)'/)?.[1];
		expect(body).toBeDefined();
		expect(JSON.parse(body!)).toEqual({
			args: [
				{
					command: "sh",
					args: ["-c", "echo hello"],
					options: { captureStdio: true },
				},
			],
		});
		expect(getAgentOsPrompt("http", code)).toContain(code);
	});

	it("omits the token from both examples when there is none", () => {
		const config = {
			apiUrl: "http://localhost:6420/",
			namespace: "default",
			token: "",
		};
		const typescript = getAgentOsTypeScriptExample(config);
		expect(typescript).not.toContain("token:");
		expect(typescript).toContain('namespace: "default",\n});');
		const curl = getAgentOsCurlExample(config);
		expect(curl).not.toContain("x-rivet-token");
		expect(curl).toMatch(
			/^curl -X POST '[^']+' \\\n {2}-H 'x-rivet-encoding: json'/,
		);
		expect(
			getAgentOsPrompt("http", curl, { includesToken: false }),
		).not.toContain("private Rivet token");
	});

	it("copies only the selected example with the docs link", () => {
		const prompt = getAgentOsPrompt("typescript", "const result = true;");
		expect(prompt).toContain(AGENT_OS_INSTALL_COMMAND);
		expect(prompt).toContain("const result = true;");
		expect(prompt).toContain("https://rivet.dev/agentos/docs/");
		expect(getAgentOsPrompt("http", "POST /gateway")).not.toContain(
			AGENT_OS_INSTALL_COMMAND,
		);
	});
});
