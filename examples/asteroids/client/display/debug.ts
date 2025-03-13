import Player, { PLAYER_CONSTS } from "../../shared/player";
import ClientGameState from "../client-gamestate";
import GameClient, { getThisPlayer } from "../state";

// @ts-ignore
const SHOW_DEBUG = import.meta.env.DEV && import.meta.env.VITE_SHOW_DEBUG === "true";

const ANGLE_NAMES = {
    0: "E",
    1: "SE",
    2: "S",
    3: "SW",
    4: "W",
    5: "NW",
    6: "N",
    7: "NE",
};

export function playerStats(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    player: Player,
    game: ClientGameState,
): { x: number; y: number } {
    const speedSq = player.velX * player.velX + player.velY * player.velY;
    const speed = Math.round(Math.sqrt(speedSq));

    const normalizedAngle = ((player.angle % (Math.PI * 2)) + Math.PI * 2) % (Math.PI * 2);
    const facingIdx = Math.round((normalizedAngle / Math.PI) * 4) % 8;
    const facing = ANGLE_NAMES[facingIdx];
    const invicibilityTime = Math.max(player.invincibilityTimeLeft, 0).toFixed(2);

    const shotTime = player.lastShot - game.clientGameState.physicsTime + PLAYER_CONSTS.SECONDS_BETWEEN_SHOTS;
    const shotTimeMs = Math.max(0, Math.round(shotTime * 1000))
        .toString()
        .padStart(3, "0");

    ctx.fillStyle = "white";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";

    ctx.font = "bold 1.5rem monospace";
    ctx.fillText("Player", x, y);

    ctx.font = "bold 1rem monospace";
    ctx.fillText(`Speed:     ${speed}`, x + 20, y + 30);
    ctx.fillText(`Facing:    ${facing}`, x + 20, y + 45);
    ctx.fillText(`Invinc.:   ${invicibilityTime}s`, x + 20, y + 60);
    ctx.fillText(`Next Shot: ${shotTimeMs}ms`, x + 20, y + 90);

    return { x: x, y: y + 115 };
}

export function gameStats(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    game: ClientGameState,
): { x: number; y: number } {
    const asteroids = Object.values(game.clientGameState.asteroids).length;
    const bullets = Object.values(game.clientGameState.bullets).length;
    const players = Object.values(game.clientGameState.players).length;
    const isPlaying = game.running ? "Yes" : "No";

    ctx.fillStyle = "white";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";

    ctx.font = "bold 1.5rem monospace";
    ctx.fillText("Game", x, y);

    ctx.font = "bold 1rem monospace";
    ctx.fillText(`Asteroids: ${asteroids}`, x + 20, y + 30);
    ctx.fillText(`Bullets:   ${bullets}`, x + 20, y + 45);
    ctx.fillText(`Players:   ${players}`, x + 20, y + 60);
    ctx.fillText(`Running:   ${isPlaying}`, x + 20, y + 75);

    return { x: x, y: y + 100 };
}

export function syncStats(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    gameState: ClientGameState,
): { x: number; y: number } {
    const now = window.performance.now();

    const lastSync = gameState.serverSync;
    const msSinceSync = Math.round(now - lastSync)
        .toString()
        .padStart(2, "0");

    ctx.fillStyle = "white";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";

    ctx.font = "bold 1.5rem monospace";
    ctx.fillText("Server Sync", x, y);

    ctx.font = "bold 1rem monospace";
    ctx.fillText(`Last Sync: ${msSinceSync}ms ago`, x + 20, y + 30);

    return { x: x, y: y + 90 };
}

export default function debug(ctx: CanvasRenderingContext2D, client: GameClient) {
    if (!SHOW_DEBUG) return;

    ctx.save();

    let x = 10;
    let y = (client.screenSize.h * 2) / 3;

    const player = getThisPlayer(client);
    const gameState = client.game;

    if (gameState && player) {
        const { x: x2, y: y2 } = playerStats(ctx, x, y, player, gameState);

        x = x2;
        y = y2;
    }

    if (gameState) {
        const { x: x2, y: y2 } = gameStats(ctx, x, y, gameState);

        x = x2;
        y = y2;
    }
    if (gameState) {
        const { x: x2, y: y2 } = syncStats(ctx, x, y, gameState);

        x = x2;
        y = y2;
    }

    ctx.restore();
}
