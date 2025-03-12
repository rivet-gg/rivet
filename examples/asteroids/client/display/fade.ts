export default function fade(
    ctx: CanvasRenderingContext2D,
    color: { r: number; g: number; b: number },
    opacity: number,
    screenSize: { w: number; h: number },
) {
    ctx.save();

    ctx.fillStyle = `rgba(${color.r} ${color.g} ${color.b} / ${opacity})`;
    ctx.fillRect(0, 0, screenSize.w, screenSize.h);

    ctx.restore();
}
