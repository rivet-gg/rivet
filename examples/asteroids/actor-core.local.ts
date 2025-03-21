import { setup } from "actor-core";
import { serve } from "@actor-core/nodejs";
import { lobbyManager } from "@actor-core/lobby-manager";

const app = setup({
	actors: {
		lobbyManager: lobbyManager({
			lobbies: {
				backend: {
					rivet: {
						ports: {
							websocket: {
								protocol: "http",
								port: 3000,
							},
						},
						resources: {
							cpu: 500,
							memory: 512,
						},
					},
					//localDevelopment: {
					//	ports: {
					//		websocket: {
					//			protocol: "http",
					//			port: 3000,
					//		},
					//	},
					//},
				},
			},
			players: {},
			rivet: {
				token: process.env.RIVET_SERVICE_TOKEN,
				project: "asteroids-vk2",
				environment: "prod",
			},
		}),
	},
});

serve(app);
