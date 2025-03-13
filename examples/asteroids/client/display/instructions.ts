// import { getZoomValue } from "../screensize";

import { getTextScale } from "../screensize";

export default function instructions(
    ctx: CanvasRenderingContext2D,
    screenSize: { w: number; h: number },
    screenScale: number,
) {
    ctx.save();

    ctx.fillStyle = "white";
    ctx.textAlign = "center";
    ctx.textBaseline = "bottom";
    ctx.font = '500 1.5rem "Turret Road"';
    ctx.translate(screenSize.w / 2, screenSize.h - 10);
    ctx.scale(getTextScale(screenScale), getTextScale(screenScale));

    ctx.fillText("[ Click to shoot ]", 0, 0);
    ctx.fillText("[ Use mouse to conrol ship ]", 0, -40);

    ctx.restore();
}
