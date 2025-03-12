import { setup } from "actor-core";
import { lobbyManager } from "@actor-core/lobby-manager";
import { RIVET_SERVICE_TOKEN } from "./token.secret";

export const app = setup({
	actors: {
		lobbyManager: lobbyManager({
			lobbies: {
				regions: ["atl"],
				backend: {
					rivet: {
						ports: {
							websocket: {
								protocol: "https",
							},
						},
						resources: {
							cpu: 500,
							memory: 512,
						},
					},
				},
			},
			players: {},
			rivet: {
				token: RIVET_SERVICE_TOKEN,
				project: "asteroids-vk2",
				environment: "test",
			},
		}),
	},
	cors: { origin: ["https://hub.rivet.gg", "http://localhost:5173"] },
});

export type App = typeof app;
