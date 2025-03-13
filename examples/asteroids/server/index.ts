import dotenv from "dotenv-flow";
dotenv.config();

import Connection, {
	checkForDeath,
	checkForSever,
	createConnection,
} from "./connection";
import { ready as lobbyReady } from "./rivet";
import { Client } from "actor-core/client";
import type { Matchmaker } from "@actor-core/matchmaker";

import { Server } from "socket.io";
import { ServerSideSocketServer, serverConfig } from "@shared/socket";
import {
	asteroids,
	ensureAsteroidCount,
	newBulletHellGame,
	newRandomGame,
	players,
	updateGame,
} from "../shared/gamestate";
import { applyPlayerInput } from "../shared/player";

const PHYSICS_UPDATES_PER_SECOND = 60;
const PHYSICS_UPDATES_PER_MESSAGE = 6;

async function main() {
	const gameModeName = process.env.GAME_MODE ?? "default";
	const actorCoreEndpoint = process.env.ACTOR_CORE_ENDPOINT;
	const lobbyToken = process.env.LOBBY_TOKEN;
	const wsPort = process.env.WEBSOCKET_PORT;
	if (!actorCoreEndpoint) throw new Error("Missing ACTOR_CORE_ENDPOINT");
	if (!wsPort) throw new Error("Missing WEBSOCKET_PORT");

	function getGameModeGame() {
		if (gameModeName === "default")
			return newRandomGame({ x: 2500, y: 2500 }, 45);
		if (gameModeName === "bullet-hell")
			return newBulletHellGame({ x: 2500, y: 2500 }, 200);
		throw new Error("Invalid game mode name.");
	}

	console.log(`Starting ${gameModeName} lobby...`);

	const globalGame = getGameModeGame();

	const actorCore = new Client(actorCoreEndpoint);
	const matchmaker = await actorCore.get<Matchmaker>({
		name: "matchmaker",
	});

	const connections = new Set<Connection>();

	const socketServer: ServerSideSocketServer = new Server(
		Number.parseInt(wsPort),
		serverConfig,
	);

	socketServer.on("connection", (sock) => {
		const takenNames = new Set<string>();
		for (const connection of connections)
			takenNames.add(connection.lifetime.playerName);
		connections.add(
			createConnection(
				sock,
				globalGame,
				takenNames,
				matchmaker,
				lobbyToken,
			),
		);
	});

	console.log("Websocket server initialized");

	const dt = 1 / PHYSICS_UPDATES_PER_SECOND;
	const dtUpdate = dt * PHYSICS_UPDATES_PER_MESSAGE;

	setInterval(() => {
		for (const connection of connections) {
			if (!connection.stateful.stopped) {
				connection.lifetime.socket.emit("update", {
					state: globalGame,
					timestamp: Date.now(),
				});
			}
		}
	}, 1000 * dtUpdate);

	setInterval(() => {
		for (const player of players(globalGame)) {
			applyPlayerInput(player, dt);
		}
		updateGame(globalGame, dt, "");
		if (
			globalGame.targetAsteroids === 0 &&
			asteroids(globalGame).length === 0
		) {
			for (const connection of connections) {
				connection.lifetime.socket.emit("stopUpdates", {});
				connection.lifetime.socket.disconnect();
			}
			console.log("Bullet hell won");
			process.exit(0);
		}
		ensureAsteroidCount(globalGame);

		for (const connection of connections) {
			// If the client has left, then delete the connection
			if (connection.stateful.disconnected) {
				connections.delete(connection);
				continue;
			}
			checkForDeath(connection);
			checkForSever(connection);
		}
	}, 1000 * dt);

	await matchmaker.setLobbyReady({ lobbyToken }).catch((err) => {
		console.error("Error starting lobby:", err);
		process.exit(1);
	});
	console.log("Lobby ready");
}

main();
