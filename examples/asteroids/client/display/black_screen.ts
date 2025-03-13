import { getTextScale } from "../screensize";

export default function blackScreen(
    ctx: CanvasRenderingContext2D,
    main: string,
    sub: string,
    screenSize: { w: number; h: number },
    screenScale: number,
) {
    // Save at initial state
    ctx.save();

    // Black out the screen
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, screenSize.w, screenSize.h);

    // Set up text style
    ctx.fillStyle = "white";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";

    ctx.translate(screenSize.w / 2, screenSize.h / 2 - 40);
    ctx.scale(getTextScale(screenScale), getTextScale(screenScale));

    // Draw first line to screen
    ctx.font = 'normal 5em "Turret Road"';
    ctx.fillText(main, 0, 0);

    // Draw second line to screen
    ctx.font = 'normal 3em "Turret Road"';
    ctx.fillText(sub, 0, 60);

    // Restore to initial state
    ctx.restore();
}
