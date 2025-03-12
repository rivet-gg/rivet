import Bullet, { BULLET_CONSTS } from "@shared/bullet";

const DRAW_SIZE_MULTIPLIER = 0.5;

export default function drawBullet(ctx: CanvasRenderingContext2D, bullet: Bullet, thisPlayerId: string) {
    ctx.save();
    ctx.translate(bullet.posX, bullet.posY);

    ctx.fillStyle = thisPlayerId === bullet.playerId ? "cyan" : "red";

    const drawSize = BULLET_CONSTS.SIZE * DRAW_SIZE_MULTIPLIER;

    ctx.beginPath();
    ctx.ellipse(0, 0, drawSize, drawSize, 0, 0, Math.PI * 2);
    ctx.fill();

    ctx.restore();
}
