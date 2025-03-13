import { serve } from "@actor-core/nodejs";
import { matchmaker } from "@actor-core/matchmaker";

serve({
	actors: {
		matchmaker: matchmaker({
			lobbies: {
				regions: ["atl"],
				backend: {
					localDevelopment: {
						ports: {
							websocket: {
								protocol: "http",
								port: 3000,
							},
						},
					},
				},
			},
			players: {},
		}),
	},
	//cors: { origin: ["https://hub.rivet.gg"] }
});
