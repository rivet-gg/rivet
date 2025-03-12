import Player from "@shared/player";
import ClientGameState from "../client-gamestate";
import Score, { calcScore, orderPlayersByScoreDesc } from "@shared/score";

import drawGameObjects from "./game/wholegame";
import grid from "./grid";
import score from "./score";
import scoreboard from "./scoreboard";
import instructions from "./instructions";

export default function game(
    ctx: CanvasRenderingContext2D,
    game: ClientGameState,
    
    centerOn: { x: number, y: number },
    selfId: string,
    scoreValue: Score,

    screenSize: { w: number; h: number },
    screenScale: number,
): void {
    // Save at initial state
    ctx.save();

    // Black out the screen
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, screenSize.w, screenSize.h);

    {
        // Scale & translate the screen to center around (0, 0)
        ctx.translate(screenSize.w / 2, screenSize.h / 2);

        // Translate again to center around the player
        ctx.translate(-centerOn.x, -centerOn.y);

        // Draw level-relative objects
        grid(ctx, game);
        drawGameObjects(ctx, game.clientGameState, selfId);
    }
    // Restore to initial state
    ctx.restore();

    scoreboard(ctx, orderPlayersByScoreDesc(game.clientGameState), screenSize, screenScale);
    score(ctx, calcScore(scoreValue), screenScale);
    instructions(ctx, screenSize, screenScale);
}
