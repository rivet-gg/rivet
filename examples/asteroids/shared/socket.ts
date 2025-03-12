import { Socket as ServerSocket, Server as SocketServer } from "socket.io";
import { Socket as ClientSocket } from "socket.io-client";

import { PlayerInput } from "./player";
import GameState from "./gamestate";

type SocketMessage<Events extends Record<string, unknown>> = {
    [key in keyof Events]: (data: Events[key]) => void;
};
type SocketMessageWithAck<Events extends Record<string, unknown>, ReturnValue> = {
    [key in keyof Events]: (data: Events[key], ack: (a: ReturnValue) => void) => void;
};

export type Input = {
    playerId: string;
    directional: PlayerInput;
    physicsTime: number;
};
export type Shoot = {
    playerId: string;
    physicsTime: number;
};
export type Respawn = {
    playerId: string;
};

export type FromClient = SocketMessageWithAck<{ input: Input }, void> &
    SocketMessageWithAck<{ shoot: Shoot }, string> &
    SocketMessageWithAck<{ respawn: Respawn }, Init>;

export type Init = {
    playerId: string;
    state: GameState;
};
export type Update = {
    timestamp: number;
    state: GameState;
};
export type EndLife = {
    timestamp: number;
    state: GameState;
};
export type StopUpdates = Record<string, never>;

export type FromServer = SocketMessage<{
    update: Update;
    endLife: EndLife;
    stopUpdates: StopUpdates;
}> &
    SocketMessageWithAck<{ init: Init }, string>;

export type ClientSideSocket = ClientSocket<FromServer, FromClient>;
export type ServerSideSocket = ServerSocket<FromClient, FromServer>;
export type ServerSideSocketServer = SocketServer<FromClient, FromServer>;


import { ServerOptions } from "socket.io";
import { ManagerOptions, SocketOptions } from "socket.io-client";

export const serverConfig: Partial<ServerOptions> = {
    transports: ["websocket"],
    allowEIO3: true,
    cors: { origin: "*" },
    perMessageDeflate: {
        thresholdSize: 256,
    },
};

export const joinConfig: Partial<SocketOptions & ManagerOptions> = {
    perMessageDeflate: serverConfig.perMessageDeflate as any,
};
