import { nanoid } from "nanoid";
import Bullet, { BULLET_CONSTS } from "./bullet";
import GameState, { asteroids, bullets, getClearSpawnpoint, isCoOp, players } from "./gamestate";
import Score from "./score";
import { clamp, delerp, lerp, lerpAngle, polarX, polarY } from "./utils";

export const PLAYER_CONSTS = {
    SIZE: 25,
    SECONDS_BETWEEN_SHOTS: 0.15,

    SPEED: 500.0,
    TURNING_SPEED: Math.PI * 1.5,
    BULLET_SPEED: 1000.0,
};

interface PlayerCollisionInfo {
    didHitAsteroid: boolean;
    hitBullets: string[];
    didHitPlayer: boolean;
}

export default interface Player {
    id: string;
    name: string;

    posX: number;
    posY: number;

    velX: number;
    velY: number;

    angle: number;

    dead: boolean;
    score: Score;

    lastShot: number;
    collisionInfo: PlayerCollisionInfo;

    playerInput: PlayerInput;
    invincibilityTimeLeft: number;
}

export function newRandomPlayer(id: string, name: string, game: GameState, tries?: number): Player {
    const { x, y } = getClearSpawnpoint(game, PLAYER_CONSTS.SIZE * 1.5, tries);

    const angle = Math.random() * Math.PI * 2;

    return {
        id,
        name,

        posX: x,
        posY: y,

        velX: 0,
        velY: 0,

        angle,

        dead: false,

        score: {
            asteroids: 0,
            players: 0,
        },

        lastShot: -PLAYER_CONSTS.SECONDS_BETWEEN_SHOTS,
        collisionInfo: {
            hitBullets: [],
            didHitAsteroid: false,
            didHitPlayer: false,
        },

        playerInput: {
            angle,
            speedScalar: 0,
        },
        invincibilityTimeLeft: 5,
    };
}

function updatePlayerCollisions(player: Player, game: GameState) {
    player.collisionInfo.didHitAsteroid = false;
    player.collisionInfo.didHitPlayer = false;
    player.collisionInfo.hitBullets = [];

    if (!isCoOp(game)) {
        for (const bullet of bullets(game)) {
            if (bullet.playerId === player.id) continue;

            const diffX = bullet.posX - player.posX;
            const diffY = bullet.posY - player.posY;
            const minDist = PLAYER_CONSTS.SIZE + BULLET_CONSTS.SIZE;

            if (diffX ** 2 + diffY ** 2 < minDist ** 2) {
                player.collisionInfo.hitBullets.push(bullet.id);
            }
        }
    }

    for (const asteroid of asteroids(game)) {
        const diffX = asteroid.posX - player.posX;
        const diffY = asteroid.posY - player.posY;
        const minDist = PLAYER_CONSTS.SIZE + asteroid.size;

        if (diffX ** 2 + diffY ** 2 < minDist ** 2) {
            player.collisionInfo.didHitAsteroid = true;
            break;
        }
    }
    for (const otherPlayer of players(game)) {
        if (otherPlayer.id === player.id) continue;
        if (player.invincibilityTimeLeft > 0 || otherPlayer.invincibilityTimeLeft > 0) continue;

        const diffX = otherPlayer.posX - player.posX;
        const diffY = otherPlayer.posY - player.posY;
        const minDist = PLAYER_CONSTS.SIZE + PLAYER_CONSTS.SIZE;

        if (diffX ** 2 + diffY ** 2 < minDist ** 2) {
            player.collisionInfo.didHitPlayer = true;
            break;
        }
    }
}

function limitAtWalls(player: Player, game: GameState) {
    const minPosX = PLAYER_CONSTS.SIZE;
    const maxPosX = game.size.w - PLAYER_CONSTS.SIZE;

    const minPosY = PLAYER_CONSTS.SIZE;
    const maxPosY = game.size.h - PLAYER_CONSTS.SIZE;

    // Limit player position to within the map
    player.posX = Math.max(minPosX, Math.min(player.posX, maxPosX));
    player.posY = Math.max(minPosY, Math.min(player.posY, maxPosY));
}

export function preUpdatePlayer(player: Player, game: GameState, dt: number) {
    updatePlayerCollisions(player, game);

    if (!player.playerInput.speedScalar) {
        player.velX *= 0.01 ** dt;
        player.velY *= 0.01 ** dt;
    }
}

export function updatePlayer(player: Player, game: GameState, dt: number) {
    const isInvincible = player.invincibilityTimeLeft > 0;

    if (player.collisionInfo.hitBullets.length > 0 && !isInvincible) {
        const finalBulletIdx = Math.floor(Math.random() * player.collisionInfo.hitBullets.length);
        const finalBulletId = player.collisionInfo.hitBullets[finalBulletIdx];
        const finalBullet = game.bullets[finalBulletId];

        if (!finalBullet) return;

        const finalPlayer = game.players[finalBullet.playerId];
        if (!finalPlayer) return;

        finalPlayer.score.players++;
        player.dead = true;

        return;
    }
    const dieFromAsteroid = player.collisionInfo.didHitAsteroid && !isInvincible;
    const dieFromNonBullet = dieFromAsteroid || player.collisionInfo.didHitPlayer;
    if (dieFromNonBullet) {
        player.dead = true;
        return;
    }

    player.posX += player.velX * dt;
    player.posY += player.velY * dt;

    limitAtWalls(player, game);

    // Not having to send internal collision information should make payload
    // sizes smaller
    player.collisionInfo = {
        hitBullets: [],
        didHitAsteroid: false,
        didHitPlayer: false,
    };

    if (player.invincibilityTimeLeft > 0) player.invincibilityTimeLeft -= dt;
}

export function canShootBullet(player: Player, game: GameState): boolean {
    return player.lastShot + PLAYER_CONSTS.SECONDS_BETWEEN_SHOTS < game.physicsTime;
}
export function shootBullet(player: Player, game: GameState): Bullet | null {
    const bullet: Bullet = {
        id: nanoid(16),
        playerId: player.id,

        posX: player.posX + Math.cos(player.angle) * PLAYER_CONSTS.SIZE,
        posY: player.posY + Math.sin(player.angle) * PLAYER_CONSTS.SIZE,

        velX: Math.cos(player.angle) * PLAYER_CONSTS.BULLET_SPEED,
        velY: Math.sin(player.angle) * PLAYER_CONSTS.BULLET_SPEED,

        spawnPhysicsTime: game.physicsTime,
        willCollide: false,
        dead: false,
    };

    const wasInvincible = player.invincibilityTimeLeft > 0;

    player.lastShot = game.physicsTime;
    player.invincibilityTimeLeft = 0;

    if (wasInvincible) return null;
    else return bullet;
}

export function clonePlayer(player: Player): Player {
    return {
        ...player,
        collisionInfo: {
            ...player.collisionInfo,
        },
    };
}

export function applyPlayerInput(player: Player, dt: number) {
    // Turning
    const t = clamp(0, dt * 5, 1);
    player.angle = lerpAngle(player.angle, player.playerInput.angle, t);

    // Acceleration
    if (player.playerInput.speedScalar) {
        const t = clamp(0, dt * 8, 1);
        const newVelX = lerp(
            player.velX,
            polarX(player.angle, PLAYER_CONSTS.SPEED * player.playerInput.speedScalar),
            t,
        );
        const newVelY = lerp(
            player.velY,
            polarY(player.angle, PLAYER_CONSTS.SPEED * player.playerInput.speedScalar),
            t,
        );

        player.velX = newVelX;
        player.velY = newVelY;
    }
}

export function getPlayerInputForMouseLocation(mouseX: number, mouseY: number): PlayerInput {
    const minDimension = Math.min(window.innerWidth, window.innerHeight);

    const diffX = mouseX - window.innerWidth / 2;
    const diffY = mouseY - window.innerHeight / 2;
    const dist = Math.hypot(diffX, diffY);

    const speedScalar = clamp(0, delerp(minDimension / 8, minDimension / 5, dist), 1);

    const angle = Math.atan2(diffY, diffX);

    return { angle, speedScalar };
}

export interface PlayerInput {
    angle: number;
    speedScalar: number;
}
