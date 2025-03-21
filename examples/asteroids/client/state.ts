import Player, {
	PlayerInput,
	canShootBullet,
	shootBullet,
} from "../shared/player";
import ClientGameState, { update } from "./client-gamestate";
import Connection, {
	createConnection,
	sendInput,
	sendRespawn,
	sendShoot,
	setupListeners,
} from "./connection";
import debug from "./display/debug";
import blackScreen from "./display/black_screen";
import game from "./display/game";
import { resizeClient } from "./screensize";
import ParticleSet, { PARTICLE_TIME, fromPlayer } from "@shared/particles";
import fade from "@display/fade";
import { ActorHandle, Client, createClient } from "actor-core/client";
import type { LobbyManager } from "@actor-core/lobby-manager";
import type { App } from "../actor-core";

export default interface GameClient {
	findingServer: boolean;
	dying: boolean;
	lobbyAddress: string | null;
	playerToken: string | null;
	connection: Connection | null;

	game: ClientGameState | null;
	id: string | null;

	screenSize: { w: number; h: number };
	screenScale: number;
	canvas: HTMLCanvasElement;
	performance: Performance;

	actorCore: Client<App>;
	lobbyManager: ActorHandle<LobbyManager>;
}

export async function initClient(canvasId: string): Promise<GameClient> {
	const actorCoreEndpoint = process.env.ACTOR_CORE_ENDPOINT;
	if (!actorCoreEndpoint) throw new Error("Missing ACTOR_CORE_ENDPOINT");

	const actorCore = createClient<App>(actorCoreEndpoint);
	const lobbyManager = await actorCore.lobbyManager.get();

	const canvas = document.querySelector<HTMLCanvasElement>(
		`canvas#${canvasId}`,
	);

	if (!canvas) {
		const canvasErrMessage = `Can not find canvas with ID ${canvasId}`;
		const canvasErrHelp =
			"Make sure this function is being called after the canvas has been loaded.";
		throw new Error(`${canvasErrMessage}\n${canvasErrHelp}`);
	}

	const client: GameClient = {
		game: null,
		connection: null,
		playerToken: null,
		lobbyAddress: null,
		findingServer: false,
		dying: false,
		screenSize: {
			w: window.innerWidth, // * window.devicePixelRatio,
			h: window.innerHeight, // * window.devicePixelRatio,
		},
		screenScale: 1,

		performance: window.performance,

		canvas,

		actorCore,
		lobbyManager,

		id: null,
	};

	resizeClient(client);

	return client;
}

export async function setup(client: GameClient) {
	const gameMode = sessionStorage.getItem("gamemode") ?? "default";
	sessionStorage.setItem("gamemode", gameMode);

	client.findingServer = true;

	const res = await client.lobbyManager.findOrCreateLobby({
		//query: { version: "local", tags: { gameMode } },
		query: { version: "local" },
		players: [{}],
		noWait: false,
		lobby: {
			maxPlayers: 8,
			maxPlayersDirect: 8,
			version: "foo",
			region: "lax",
		},
	});

	let lobbyAddress: string;
	let playerToken: string;
	if ("localDevelopment" in res.lobby.backend) {
		const port = res.lobby.backend.localDevelopment.ports.websocket;
		if (!port) throw new Error("Missing port");
		if (!res.players[0]) throw new Error("Missing player");

		lobbyAddress = `${port.protocol}://${port.hostname}:${port.port}`;
		playerToken = res.players[0].token;

		client.lobbyAddress = lobbyAddress;
		client.playerToken = playerToken;
	} else if ("rivet" in res.lobby.backend) {
		const port = res.lobby.backend.rivet.ports?.websocket;
		if (!port) throw new Error("Missing port");
		if (!res.players[0]) throw new Error("Missing player");

		lobbyAddress = `${port.protocol}://${port.hostname}:${port.port}`;
		playerToken = res.players[0].token;

		client.lobbyAddress = lobbyAddress;
		client.playerToken = playerToken;
	} else {
		throw "UNIMPLEMENTED";
	}

	client.connection = createConnection(
		lobbyAddress,
		playerToken,
		client.performance.now(),
	);
	setupListeners(client.connection, client, client.performance);
	client.connection.socket.connect();
}
export async function respawn(client: GameClient) {
	if (getClientState(client) !== ClientState.DEAD) return;
	if (!client.connection) return location.reload();

	sendRespawn(client.connection, client);
}

export async function setPlayerInput(client: GameClient, input: PlayerInput) {
	const connection = client.connection;
	const game = client.game;
	const playerId = client.id;

	if (!connection)
		throw new Error(
			"Connection must be initialized before setting player input.",
		);
	if (!game || !game.running) {
		console.log("Game not running, cancelling player input...");
		return;
	}
	if (!playerId) {
		console.log("Player ID not confirmed, cancelling player input...");
		return;
	}
	const clientPlayer = game.clientGameState.players[playerId];
	const serverPlayer = game.serverGameState.players[playerId];

	if (!clientPlayer || !serverPlayer) {
		client.dying = true;
		delete game.clientGameState.players[playerId];
		delete game.serverGameState.players[playerId];
	}

	await sendInput(
		connection,
		playerId,
		{ ...input },
		client.performance.now(),
	);
	clientPlayer.playerInput = { ...input };
	serverPlayer.playerInput = { ...input };
}

export async function tryShoot(client: GameClient) {
	const connection = client.connection;
	const game = client.game;
	const playerId = client.id;

	if (!connection)
		throw new Error(
			"Connection must be initialized before shooting a bullet.",
		);
	if (!game) {
		console.log("Game not initialized, cancelling player bullet...");
		return;
	}
	if (!playerId) {
		console.log("Player ID not confirmed, cancelling player bullet...");
		return;
	}

	const player = game.clientGameState.players[playerId];

	if (canShootBullet(player, game.clientGameState)) {
		const id = await sendShoot(
			connection,
			playerId,
			game.clientGameState.physicsTime,
		);
		if (!id) return;

		const clientBullet = shootBullet(player, game.clientGameState);
		const serverBullet = shootBullet(player, game.serverGameState);
		if (!clientBullet || !serverBullet) return;

		clientBullet.id = id;
		serverBullet.id = id;

		game.clientGameState.bullets[clientBullet.id] = clientBullet;
		game.serverGameState.bullets[serverBullet.id] = serverBullet;
	}
}

export function drawScreen(client: GameClient) {
	const scale = client.screenScale;
	const size = client.screenSize;

	const ctx = client.canvas.getContext("2d");
	if (!ctx)
		throw new Error("Unable to get a 2d context for the target canvas");
	ctx.save();
	ctx.scale(client.screenScale, client.screenScale);
	ctx.imageSmoothingQuality = "low";

	const state = getClientState(client);
	const player = getThisPlayer(client);
	const particleSet = getThisParticleSet(client);

	switch (state) {
		case ClientState.INITIAL:
			// Idling in start state
			blackScreen(ctx, "ASTEROIDS", "[Click to start]", size, scale);
			break;

		case ClientState.FINDING_SERVER:
			// Requesting from lobby manager
			blackScreen(ctx, "Looking for server...", "", size, scale);
			break;

		case ClientState.CONNECTING:
			// Connection established, waiting for init message
			blackScreen(ctx, "Connecting...", "", size, scale);
			break;

		case ClientState.WAITING_FOR_SPAWN:
			// Socket is connected, waiting to receive client info.
			blackScreen(ctx, "Spawning...", "", size, scale);
			break;

		case ClientState.PLAYING:
			if (!client.game || !player)
				throw new Error("Unreachable condition");

			// Game running
			game(
				ctx,
				client.game,

				{ x: player.posX, y: player.posY },
				player.id,
				player.score,

				size,
				scale,
			);
			break;

		case ClientState.DEAD_IN_GAME:
			if (!client.game || !client.id || !particleSet)
				throw new Error("Unreachable condition");

			// Dying
			game(
				ctx,
				client.game,

				particleSet.displayPos,
				client.id,
				{ asteroids: 0, players: 0 },

				size,
				scale,
			);
			fade(
				ctx,
				{ r: 0, g: 0, b: 0 },
				1 - particleSet.timeLeft / PARTICLE_TIME,
				size,
			);
			break;

		case ClientState.DEAD:
			// Player is dead, waiting for respawn
			blackScreen(ctx, "You died.", "[Click to respawn]", size, scale);
			break;

		case ClientState.UNKNOWN:
			// Unknown state, reloading
			// window.location.reload();
			break;
	}

	debug(ctx, client);
	ctx.restore();
}

export function getThisPlayer(client: GameClient): Player | null {
	const playerId = client.id;
	if (!playerId) return null;

	const player = client.game?.clientGameState.players[playerId];
	if (!player) return null;

	return player;
}

function getThisParticleSet(client: GameClient): ParticleSet | null {
	const playerId = client.id;
	if (!playerId) return null;

	const particleSet = client.game?.clientGameState.particleSets[playerId];
	if (!particleSet) return null;

	return particleSet;
}

export function startClientDrawloop(client: GameClient): () => void {
	const stopDrawLoopController = new AbortController();
	const stopSignal = stopDrawLoopController.signal;

	const loop = () => {
		if (!stopSignal.aborted) {
			handleStateTransitions(client);

			if (client.connection) {
				const prevTime = client.connection.lastPhysicsUpdate ?? 0;
				const newTime = client.performance.now();
				client.connection.lastPhysicsUpdate = newTime;

				if (client.game?.running && client.id)
					update(client.game, client.id, newTime - prevTime, newTime);
			}
			drawScreen(client);
			requestAnimationFrame(loop);
		}
	};
	requestAnimationFrame(loop);

	return () => stopDrawLoopController.abort();
}

export enum ClientState {
	INITIAL,
	FINDING_SERVER,
	CONNECTING,
	WAITING_FOR_SPAWN,
	PLAYING,
	DEAD_IN_GAME,
	DEAD,

	UNKNOWN,
}

export function getClientState(client: GameClient): ClientState {
	const findingServer = client.findingServer;
	const hasLobby = !!client.lobbyAddress;
	const hasId = !!client.id;
	const hasGame = !!client.game;
	const playerExists =
		!!client.game &&
		!!client.id &&
		!!client.game.clientGameState.players[client.id];
	const particlesExist =
		!!client.game &&
		!!client.id &&
		!!client.game.clientGameState.particleSets[client.id];

	if (!findingServer) return ClientState.INITIAL;
	if (!hasLobby) return ClientState.FINDING_SERVER;
	if (!hasId) return ClientState.CONNECTING;

	if (playerExists) return ClientState.PLAYING;
	if (particlesExist) return ClientState.DEAD_IN_GAME;

	if (!hasGame) return ClientState.DEAD;
	if (hasGame && !playerExists) return ClientState.WAITING_FOR_SPAWN;

	return ClientState.UNKNOWN;
}

export function handleStateTransitions(client: GameClient) {}
