import { actor, event } from "rivetkit";
import { db } from "rivetkit/db";
import type { OpenCodeEvent } from "./host.js";
import {
	closeRuntime,
	createRuntime,
	type OpenCodeRuntime,
	requireRuntime,
} from "./runtime.js";

// One embedded OpenCode host per Rivet Actor, retained from wake until sleep.
export const opencode = actor({
	options: { actionTimeout: 2_147_483_647, sleepGracePeriod: 15 * 60_000 },
	db: db(),
	createVars: () => ({
		opencode: undefined as Promise<OpenCodeRuntime> | undefined,
	}),
	events: { event: event<OpenCodeEvent>() },
	onWake: (c) => {
		c.vars.opencode = createRuntime(c);
		return c.vars.opencode.then(() => undefined);
	},
	onSleep: closeRuntime,
	onDestroy: closeRuntime,
	actions: {
		getSession: async (c) =>
			(await requireRuntime(c.vars.opencode)).getSession(c),
		prompt: async (c, text: string) =>
			(await requireRuntime(c.vars.opencode)).prompt(c, text),
		getMessages: async (c) =>
			(await requireRuntime(c.vars.opencode)).getMessages(c),
		waitForIdle: async (c) =>
			(await requireRuntime(c.vars.opencode)).waitForIdle(c),
		abort: async (c) => (await requireRuntime(c.vars.opencode)).abort(c),
	},
});
