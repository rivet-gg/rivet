import GameState, { asteroids, isCoOp, players } from "./gamestate";
import { PLAYER_CONSTS } from "./player";
import { withinRadius } from "./utils";

export const BULLET_CONSTS = {
    SIZE: 10,
    BULLET_LIFE_TIME: 1.0,
};

export default interface Bullet {
    dead: boolean;
    willCollide: boolean;

    id: string;
    playerId: string;
    posX: number;
    posY: number;
    velX: number;
    velY: number;

    spawnPhysicsTime: number;
}

export function preUpdateBullet(bullet: Bullet, game: GameState, dt: number) {
    const pos = { x: bullet.posX, y: bullet.posY };

    for (const asteroid of asteroids(game)) {
        const colliding = withinRadius({ x: asteroid.posX, y: asteroid.posY }, pos, BULLET_CONSTS.SIZE + asteroid.size);

        if (colliding) {
            bullet.willCollide = true;
            return;
        }
    }

    if (!isCoOp(game)) {
        for (const player of players(game)) {
            if (player.id === bullet.playerId) continue;

            const colliding = withinRadius(
                { x: player.posX, y: player.posY },
                pos,
                BULLET_CONSTS.SIZE + PLAYER_CONSTS.SIZE,
            );

            if (colliding) {
                bullet.willCollide = true;
                return;
            }
        }
    }
}

export function updateBullet(bullet: Bullet, game: GameState, dt: number) {
    if (bullet.willCollide) bullet.dead = true;
    if (game.physicsTime - bullet.spawnPhysicsTime > BULLET_CONSTS.BULLET_LIFE_TIME) bullet.dead = true;

    const bulletOutOfXBounds = 0 > bullet.posX || bullet.posX > game.size.w;
    const bulletOutOfYBounds = 0 > bullet.posY || bullet.posY > game.size.h;
    if (bulletOutOfXBounds || bulletOutOfYBounds) bullet.dead = true;

    if (bullet.dead) return;

    bullet.posX += bullet.velX * dt;
    bullet.posY += bullet.velY * dt;
}

export function cloneBullet(bullet: Bullet): Bullet {
    return {
        ...bullet,
    };
}
