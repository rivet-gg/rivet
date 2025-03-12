import { getTextScale } from "../screensize";

export default function score(ctx: CanvasRenderingContext2D, score: number, screenScale: number) {
    ctx.save();
    ctx.scale(getTextScale(screenScale), getTextScale(screenScale));

    ctx.fillStyle = "white";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.font = '500 3rem "Turret Road"';
    ctx.fillText(score.toString(), 10, 10);

    ctx.restore();
}
