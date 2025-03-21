import ClientGameState from "../client-gamestate";

const HORIZ_LINE_COUNT = 10;
const VERT_LINE_COUNT = 10;

export default function grid(ctx: CanvasRenderingContext2D, game: ClientGameState) {
    const gameW = game.clientGameState.size.w;
    const gameH = game.clientGameState.size.h;

    ctx.save();

    // Map Bounding box
    ctx.strokeStyle = "white";
    ctx.lineWidth = 3;
    ctx.strokeRect(0, 0, gameW, gameH);

    // Map grid
    ctx.strokeStyle = "#333";
    ctx.lineWidth = 3;
    ctx.beginPath();

    // Vertical lines
    for (let x = 1; x < VERT_LINE_COUNT; x++) {
        const xVal = (gameW / VERT_LINE_COUNT) * x;
        ctx.moveTo(xVal, 0);
        ctx.lineTo(xVal, gameH);
    }

    // Horizontal line
    for (let y = 1; y < HORIZ_LINE_COUNT; y++) {
        const yVal = (gameH / HORIZ_LINE_COUNT) * y;
        ctx.moveTo(0, yVal);
        ctx.lineTo(gameW, yVal);
    }

    ctx.stroke();

    ctx.restore();
}
