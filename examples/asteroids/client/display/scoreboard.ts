import { ScoreboardEntry } from "../../shared/score";
import { getTextScale } from "../screensize";

export default function scoreboard(
    ctx: CanvasRenderingContext2D,
    entries: ScoreboardEntry[],
    screenSize: { w: number; h: number },
    screenScale: number,
) {
    ctx.save();

    const nameOffset = -240;
    let entryY = 10;

    ctx.translate(screenSize.w - 10, 0);
    ctx.scale(getTextScale(screenScale), getTextScale(screenScale));

    for (const { scoreNum, player } of entries) {
        ctx.font = '800 1.5rem "Turret Road"';
        ctx.fillStyle = "#FFFFFFBD ";

        ctx.textAlign = "left";
        ctx.textBaseline = "top";
        ctx.fillText(player.name, nameOffset, entryY);

        ctx.textAlign = "right";
        ctx.textBaseline = "top";
        ctx.fillText(scoreNum.toString(), 0, entryY);

        entryY += 20;
    }

    ctx.restore();
}
