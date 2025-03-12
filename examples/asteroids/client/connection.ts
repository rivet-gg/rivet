import { ClientSideSocket, Init, Update, EndLife } from "../shared/socket";

import { io } from "socket.io-client";
import { PlayerInput } from "../shared/player";
import { initClientGamestate, serverSync } from "./client-gamestate";
import GameClient from "./state";

class ConnectionError extends Error {}

export default interface Connection {
    socket: ClientSideSocket;

    createdAt: number;
    finishedSetup: number | null;
    initializedAt: number | null;
    playerSpawned: number | null;
    lastServerUpdate: number | null;
    lastPhysicsUpdate: number | null;
}

export function createConnection(lobbyAddress: string, playerToken: string, now: number): Connection {
    return {
        socket: getSocketForConnectionTarget(lobbyAddress, playerToken),

        createdAt: now,
        finishedSetup: null,
        initializedAt: null,
        playerSpawned: null,
        lastServerUpdate: null,
        lastPhysicsUpdate: null,
    };
}

function getSocketForConnectionTarget(lobbyAddress: string, playerToken: string): ClientSideSocket {
    try {
        const socket = io(lobbyAddress, {
            transports: ["websocket"],
            reconnection: true,
            query: { token: playerToken },
            autoConnect: false,
        });

        return socket;
    } catch (e) {
        throw new ConnectionError(`Could not connect to ${lobbyAddress} (error: ${e})`);
    }
}

export function setupListeners(connection: Connection, client: GameClient, performance: Performance) {
    if (connection.finishedSetup) {
        const connectionErrorStr = "This connection has already been setup";
        const connectionHelpStr = "This function should only be called once per websocket/connection.";
        throw new ConnectionError(`${connectionErrorStr}\n${connectionHelpStr}`);
    }

    connection.socket.once("init", (data: Init, callback) =>
        handleInit(client, connection, data, callback, performance.now()),
    );

    connection.socket.on("update", (data: Update) => handleUpdate(client, connection, data, performance.now()));

    connection.socket.on("endLife", (data: EndLife) => handleEndLife(client, connection, data, performance.now()));

    connection.socket.on("stopUpdates", () => handleStoppingUpdates(client, connection, performance.now()));

    connection.socket.once("disconnect", () => handleDisconnect(client, connection, performance.now()));

    connection.finishedSetup = performance.now();
}

function handleInit(client: GameClient, connection: Connection, data: Init, callback: (id: string) => void, now: number) {
    console.log(`Recieved init message @ ${client.performance.now()}`);

    if (client.game) throw new Error("Client game already exists");

    client.game = initClientGamestate(data.state, now);
    client.game.running = true;

    console.log(`Player ID: ${data.playerId}`);

    client.id = data.playerId;
    connection.initializedAt = now;

    console.log("Sending acknowledgement to server");
    callback(data.playerId);
    console.log("Sent ack message", client.performance.now());
}

function handleUpdate(client: GameClient, connection: Connection, data: Update, now: number) {
    if (!client.game) return;
    if (!client.game.running) return;

    serverSync(client.game, data.state, client.id, performance.now());
    connection.lastServerUpdate = now;

    if (client.id && data.state.players[client.id]) connection.playerSpawned = now;
}

function handleEndLife(client: GameClient, connection: Connection, data: EndLife, now: number) {
    if (client.id && client.game && client.game.running) {
        serverSync(client.game, data.state, client.id, performance.now());

        delete client.game.clientGameState.players[client.id];
        delete client.game.serverGameState.players[client.id];
    }
    
    client.dying = true;
    connection.lastServerUpdate = now;
}
function handleStoppingUpdates(client: GameClient, connection: Connection, now: number) {
    client.game = null;
    client.dying = false;
}

function handleDisconnect(client: GameClient, connection: Connection, now: number) {
    // client.connection = null;
    location.reload();
}

export async function sendRespawn(connection: Connection, client: GameClient) {
    console.log(`Requesting a respawn @ ${client.performance.now()}`);

    if (!client.id) throw new Error("Client ID not initialized");

    const respawnData = await connection.socket.emitWithAck("respawn", { playerId: client.id });

    const now = client.performance.now();
    console.log(`Respawning @ ${now}`);

    if (client.game) throw new Error("Client game already exists");
    if (client.id !== respawnData.playerId) throw new Error("Unknown Error Occured");

    client.game = initClientGamestate(respawnData.state, now);
    client.game.running = true;
    connection.initializedAt = now;
}
export async function sendInput(connection: Connection, playerId: string, input: PlayerInput, physicsTime: number) {
    return await connection.socket.emitWithAck("input", { playerId, directional: input, physicsTime });
}
export async function sendShoot(connection: Connection, playerId: string, physicsTime: number): Promise<string> {
    return await connection.socket.emitWithAck("shoot", { playerId, physicsTime });
}
