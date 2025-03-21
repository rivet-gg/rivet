import {
	ServerSideSocket,
	Input,
	Shoot,
	Respawn,
	Init,
} from "../shared/socket";
import { canShootBullet, newRandomPlayer, shootBullet } from "../shared/player";

import GameState from "../shared/gamestate";
import { sequentialNumberName } from "../shared/names";
import type { LobbyManager } from "@actor-core/lobby-manager";
import type { ActorHandle } from "actor-core/client";

// Lifetime data won't change for the entire time the player is on the page.
// However, it will not persist across page reloads.
interface LifetimeData {
	playerToken: string;
	playerId: string;
	playerName: string;

	socket: ServerSideSocket;
	game: GameState;
}
// State data will change as the player plays the game.
interface StateData {
	alive: boolean;
	stopped: boolean;
	initialized: boolean;
	disconnected: boolean;

	lastInput: number;
}

export default interface Connection {
	lifetime: LifetimeData;
	stateful: StateData;
	lobbyManager: ActorHandle<LobbyManager>;
	lobbyToken?: string;
}

export function createConnection(
	socket: ServerSideSocket,
	game: GameState,
	takenNames: Set<string>,
	lobbyManager: ActorHandle<LobbyManager>,
	lobbyToken?: string,
): Connection {
	const playerToken = socket.handshake.query.token;
	const actingPlayerToken =
		!playerToken || Array.isArray(playerToken)
			? "INVALID PLAYER"
			: playerToken;

	const lifetimeData: LifetimeData = {
		playerToken: actingPlayerToken,
		playerId: crypto.randomUUID(),
		playerName: sequentialNumberName(takenNames),

		socket,
		game,
	};

	const statefulData: StateData = {
		alive: false,
		initialized: false,
		disconnected: false,
		stopped: false,
		lastInput: -1,
	};

	const connection: Connection = {
		lifetime: lifetimeData,
		stateful: statefulData,
		lobbyManager,
		lobbyToken,
	};

	if (!playerToken || Array.isArray(playerToken)) {
		handleFatalError(connection, "Invalid player token");
		return connection;
	}

	setupSocketListeners(connection, socket);

	const id = connection.lifetime.playerId;
	console.log(
		`Socket ${socket.id} setup for player ${connection.lifetime.playerName} (${id})`,
	);

	console.log(`Sending init to ${id}`);
	socket
		.emitWithAck("init", { playerId: id, state: game })
		.then((ackData) => handleInitAck(connection, ackData));

	return connection;
}

function setupSocketListeners(
	connection: Connection,
	socket: ServerSideSocket,
) {
	socket.on("input", (data, callback) =>
		handleInput(connection, data, callback),
	);
	socket.on("shoot", (data, callback) =>
		handleShoot(connection, data, callback),
	);
	socket.once("disconnect", () => handleDisconnect(connection));
	socket.on("respawn", (data, callback) =>
		handleRespawn(connection, data, callback),
	);
}

async function handleInitAck(connection: Connection, ackData: string) {
	console.log("Received acknowledgement from client");

	const {
		stateful: { initialized },
		lifetime: { playerId, playerName, playerToken, game },
	} = connection;

	if (initialized)
		return handleFatalError(connection, "Connection already initialized");
	if (playerId !== ackData)
		return handleFatalError(connection, "Player ID tampering detected");
	if (game.players[playerId])
		return handleFatalError(connection, "Player ID already found in game");
	if (connection.stateful.disconnected)
		return handleFatalError(connection, "Socket already disconnected");

	try {
		await connection.lobbyManager.setPlayersConnected({
			lobbyToken: connection.lobbyToken,
			playerTokens: [playerToken],
		});
	} catch (e) {
		return handleFatalError(connection, e);
	}
	console.log(`Player ${playerId} join logged in Rivet`);

	game.players[playerId] = newRandomPlayer(playerId, playerName, game);
	const player = game.players[playerId];

	if (player.id !== playerId || player.name !== playerName)
		return handleFatalError(connection, "Player data issue detected.");

	console.log(`Player ${player.name} (${player.id}) added to game`);

	connection.stateful.initialized = true;
	connection.stateful.alive = true;
}

function handleInput(
	connection: Connection,
	data: Input,
	callback: () => void,
) {
	const {
		stateful: { initialized, alive, lastInput, disconnected },
		lifetime: { playerId, game },
	} = connection;

	if (!initialized || !alive || disconnected) return;
	if (playerId !== data.playerId)
		return handleFatalError(connection, "Player ID tampering detected");

	if (data.physicsTime < lastInput) return;
	connection.stateful.lastInput = data.physicsTime;

	const player = game.players[playerId];
	if (!player) return;

	player.playerInput = data.directional;

	callback();
}

function handleShoot(
	connection: Connection,
	data: Shoot,
	callback: (s: string) => void,
) {
	const {
		stateful: { initialized, alive, disconnected },
		lifetime: { playerId, game },
	} = connection;

	if (!initialized || !alive || disconnected) return;
	if (playerId !== data.playerId)
		return handleFatalError(connection, "Player ID tampering detected");

	const player = game.players[playerId];
	if (!player) return;

	if (canShootBullet(player, game)) {
		const bullet = shootBullet(player, game);
		if (!bullet) return callback("");
		game.bullets[bullet.id] = bullet;
		callback(bullet.id);
	} else callback("");
}

function handleRespawn(
	connection: Connection,
	data: Respawn,
	callback: (s: Init) => void,
) {
	const {
		stateful: { initialized, alive, disconnected, stopped },
		lifetime: { playerId, playerName, game },
	} = connection;

	if (!initialized || !stopped || alive || disconnected) return;
	if (playerId in game.players || playerId in game.particleSets) return;
	if (playerId !== data.playerId)
		return handleFatalError(connection, "Player ID tampering detected");

	game.players[playerId] = newRandomPlayer(playerId, playerName, game, 10);

	connection.stateful.alive = true;
	connection.stateful.stopped = false;
	console.log(
		`Player ${connection.lifetime.playerName} (${connection.lifetime.playerId}) added to game`,
	);

	callback({ playerId, state: game });
}

export function checkForDeath(connection: Connection) {
	if (!connection.stateful.alive) return;

	const playerStillAlive =
		connection.lifetime.playerId in connection.lifetime.game.players;
	if (!playerStillAlive) {
		console.log(
			`Alerting ${connection.lifetime.playerName} (${connection.lifetime.playerId}) that they are no longer alive`,
		);

		connection.lifetime.socket.emit("endLife", {
			timestamp: Date.now(),
			state: connection.lifetime.game,
		});

		connection.stateful.alive = false;
	}
}

export function checkForSever(connection: Connection) {
	if (!connection.stateful.initialized) return;
	if (connection.stateful.alive || connection.stateful.stopped) return;

	const playerParticlesStillExist =
		connection.lifetime.playerId in connection.lifetime.game.particleSets;

	if (!playerParticlesStillExist) {
		console.log(
			`Stopping updates to ${connection.lifetime.playerName} (${connection.lifetime.playerId})`,
		);

		connection.lifetime.socket.emit("stopUpdates", {});

		connection.stateful.stopped = true;
	}
}

function handleDisconnect(connection: Connection) {
	const {
		stateful: { initialized, alive },
		lifetime: { playerId, playerName, playerToken, game, socket },
	} = connection;

	connection.stateful.disconnected = true;
	console.log(`Handling disconnect from ${playerId}`);

	console.log(`Removing ${playerName} (${playerId}) from the game`);
	delete game.players[playerId];

	if (!initialized || !alive) {
		connection.lobbyManager.setPlayersDisconnected({
			lobbyToken: connection.lobbyToken,
			playerTokens: [playerToken]
		})
		console.log("Player leave logged in Rivet");
	}

	if (!socket.disconnected) socket.disconnect();
}

function handleFatalError(connection: Connection, data: unknown) {
	console.error(
		`Connection ${connection.lifetime.playerId} encounted a fatal error:`,
		data,
	);

	if (connection.lifetime.socket.connected) handleDisconnect(connection);
}
