import Player, { PLAYER_CONSTS } from "@shared/player";

const INVINCIBILITY_BLINK_SPEED = 0.75;
const INVINCIBILITY_BLINK_DUTY_CYCLE = 0.75;

function isInvisibleBecauseInvincible(player: Player) {
    const seconds = Date.now() / 1000;
    const cycleAmount = (seconds % INVINCIBILITY_BLINK_SPEED) / INVINCIBILITY_BLINK_SPEED;


    return player.invincibilityTimeLeft > 0 && cycleAmount > INVINCIBILITY_BLINK_DUTY_CYCLE;
}

export default function drawPlayer(ctx: CanvasRenderingContext2D, player: Player, playerIsSelf: boolean) {
    const { posX, posY, angle } = player;
    const drawSize = PLAYER_CONSTS.SIZE * 1.25;

    if (isInvisibleBecauseInvincible(player)) return;

    ctx.save();
    ctx.translate(posX, posY);

    ctx.strokeStyle = playerIsSelf ? "cyan" : "red";
    ctx.fillStyle = playerIsSelf ? "cyan" : "red";
    ctx.lineWidth = 4;

    if (player.invincibilityTimeLeft > 0) {
        ctx.strokeStyle = playerIsSelf ? "#00FFFFBB" : "#FF0000BB";
        ctx.fillStyle = playerIsSelf ? "#00FFFFBB" : "#FF0000BB";
    }

    const { tip, left, back, right, fireLeft, fireBack, fireRight } = getPlayerDrawPoints(angle, drawSize);
    ctx.beginPath();
    ctx.moveTo(tip.x, tip.y);
    ctx.lineTo(left.x, left.y);
    ctx.lineTo(back.x, back.y);
    ctx.lineTo(right.x, right.y);
    ctx.lineTo(tip.x, tip.y);
    ctx.lineTo(left.x, left.y);
    ctx.stroke();

    const fireFlickerOn = Date.now() % 125 < 60;
    if (player.playerInput.speedScalar && fireFlickerOn) {
        ctx.beginPath();
        ctx.moveTo(fireLeft.x, fireLeft.y);
        ctx.lineTo(fireBack.x, fireBack.y);
        ctx.lineTo(fireRight.x, fireRight.y);
        ctx.stroke();
    }
    
    ctx.restore();
}

type Point = { x: number; y: number };
function getPlayerDrawPoints(
    angle: number,
    drawSize: number,
): {
    tip: Point;
    left: Point;
    back: Point;
    right: Point;

    fireLeft: Point;
    fireBack: Point;
    fireRight: Point;
} {
    const tipX = Math.cos(angle) * drawSize;
    const tipY = Math.sin(angle) * drawSize;

    const leftX = Math.cos(angle - Math.PI * 0.8) * drawSize;
    const leftY = Math.sin(angle - Math.PI * 0.8) * drawSize;

    const backX = Math.cos(angle + Math.PI) * drawSize * 0.5;
    const backY = Math.sin(angle + Math.PI) * drawSize * 0.5;

    const rightX = Math.cos(angle + Math.PI * 0.8) * drawSize;
    const rightY = Math.sin(angle + Math.PI * 0.8) * drawSize;

    const fireLeftX = Math.cos(angle - Math.PI * 0.87) * drawSize * 0.87;
    const fireLeftY = Math.sin(angle - Math.PI * 0.87) * drawSize * 0.87;

    const fireBackX = Math.cos(angle + Math.PI) * drawSize * 1.2;
    const fireBackY = Math.sin(angle + Math.PI) * drawSize * 1.2;

    const fireRightX = Math.cos(angle + Math.PI * 0.87) * drawSize * 0.87;
    const fireRightY = Math.sin(angle + Math.PI * 0.87) * drawSize * 0.87;

    return {
        tip: { x: tipX, y: tipY },
        left: { x: leftX, y: leftY },
        back: { x: backX, y: backY },
        right: { x: rightX, y: rightY },

        fireLeft: { x: fireLeftX, y: fireLeftY },
        fireBack: { x: fireBackX, y: fireBackY },
        fireRight: { x: fireRightX, y: fireRightY },
    };
}
