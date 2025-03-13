import { nanoid } from "nanoid";
import generateRandomNumbers, { newSeed } from "./prng";

import { PLAYER_CONSTS } from "./player";
import GameState, { asteroids, bullets, players } from "./gamestate";
import { lerp, polarObj, withinRadius } from "./utils";
import { BULLET_CONSTS } from "./bullet";

export const ASTEROID_CONSTS = {
    TARGET_SPEED: 100,
    MAX_SPEED: 10000,
    ROT_SPEED_RANGE: [-1, 1],

    SHAPE_POINTS: 16,
    SHAPE_RADIANS_PER_POINT: (Math.PI * 2) / 16,
    SHAPE_INSET_FACTOR: -2 / 3,
    SHAPE_OUTSET_FACTOR: 1 / 10,
};

interface AsteroidCollisionInfo {
    didHitAsteroid: boolean;
    hitBullets: string[];
    didHitPlayer: boolean;
}

export default interface Asteroid {
    id: string;
    size: number;
    dead: boolean;

    posX: number;
    posY: number;
    velX: number;
    velY: number;
    angle: number;
    rotationSpeed: number;

    collisionInfo: AsteroidCollisionInfo;
    shapeSeed: number;
}

export function newRandomAsteroid(
    sizeRange: [number, number],
    xRange: [number, number],
    yRange: [number, number],
    velMax: number,
    rotSpeedRange: [number, number],
): Asteroid {
    const size = Math.round((sizeRange[1] - sizeRange[0]) * Math.random() + sizeRange[0]);

    const posX = (xRange[1] - xRange[0]) * Math.random() + xRange[0];
    const posY = (yRange[1] - yRange[0]) * Math.random() + yRange[0];

    const velMag = Math.random() * velMax;
    const rotationSpeed = (rotSpeedRange[1] - rotSpeedRange[0]) * Math.random() + rotSpeedRange[0];

    const angle = Math.random() * Math.PI * 2;

    const velX = Math.cos(angle) * velMag;
    const velY = Math.sin(angle) * velMag;

    const id = nanoid(16);

    const shapeSeed = newSeed();
    const collisionInfo = {
        hitBullets: [],
        didHitAsteroid: false,
        didHitPlayer: false,
    };

    return {
        id,
        size,
        dead: false,
        posX,
        posY,
        velX,
        velY,
        angle,
        rotationSpeed,
        shapeSeed,
        collisionInfo,
    };
}

function updateAsteroidCollisions(asteroid: Asteroid, game: GameState) {
    asteroid.collisionInfo.hitBullets = [];
    asteroid.collisionInfo.didHitAsteroid = false;
    asteroid.collisionInfo.didHitPlayer = false;

    const pos = { x: asteroid.posX, y: asteroid.posY };
    const size = asteroid.size;

    for (const bullet of bullets(game)) {
        const colliding = withinRadius({ x: bullet.posX, y: bullet.posY }, pos, size + BULLET_CONSTS.SIZE);

        if (colliding) {
            asteroid.collisionInfo.hitBullets.push(bullet.id);
        }
    }

    for (const otherAsteroid of asteroids(game)) {
        const colliding = withinRadius(
            { x: otherAsteroid.posX, y: otherAsteroid.posY },
            pos,
            size + otherAsteroid.size,
        );

        if (colliding) {
            asteroid.collisionInfo.didHitAsteroid = true;
            break;
        }
    }
    for (const player of players(game)) {
        const colliding = withinRadius({ x: player.posX, y: player.posY }, pos, size + PLAYER_CONSTS.SIZE);

        if (colliding) {
            asteroid.collisionInfo.didHitPlayer = true;
            break;
        }
    }
}

export function preUpdateAsteroid(asteroid: Asteroid, game: GameState, dt: number) {
    updateAsteroidCollisions(asteroid, game);
}

export function updateAsteroid(asteroid: Asteroid, game: GameState, dt: number) {
    if (asteroid.collisionInfo.hitBullets.length > 0) {
        runDeathRoutine(asteroid, game);
        return;
    }

    const asteroidAboveTopBound = asteroid.posX - sizeAsteroid(asteroid) <= 0;
    const asteroidBelowBottomBound = asteroid.posX + sizeAsteroid(asteroid) >= game.size.w;

    const asteroidLeftOfLeftBound = asteroid.posY - sizeAsteroid(asteroid) <= 0;
    const asteroidRightOfRightBound = asteroid.posY + sizeAsteroid(asteroid) >= game.size.w;

    if (asteroidAboveTopBound) asteroid.velX = Math.abs(asteroid.velX);
    if (asteroidBelowBottomBound) asteroid.velX = -Math.abs(asteroid.velX);

    if (asteroidLeftOfLeftBound) asteroid.velY = Math.abs(asteroid.velY);
    if (asteroidRightOfRightBound) asteroid.velY = -Math.abs(asteroid.velY);

    asteroid.posX += asteroid.velX * dt;
    asteroid.posY += asteroid.velY * dt;
    asteroid.angle += asteroid.rotationSpeed * dt;

    // Not having to send internal collision information should make payload
    // sizes smaller
    asteroid.collisionInfo = {
        hitBullets: [],
        didHitAsteroid: false,
        didHitPlayer: false,
    };
}

export function runDeathRoutine(asteroid: Asteroid, game: GameState) {
    const finalBulletIdx = Math.floor(Math.random() * asteroid.collisionInfo.hitBullets.length);
    const finalBulletId = asteroid.collisionInfo.hitBullets[finalBulletIdx];
    const finalBullet = game.bullets[finalBulletId];
    if (!finalBullet) return;

    const finalPlayer = game.players[finalBullet.playerId];
    if (!finalPlayer) return;

    finalPlayer.score.asteroids++;

    asteroid.dead = true;
}

export function sizeAsteroid(asteroid: Asteroid): number {
    return asteroid.size;
}
export function massAsteroid(asteroid: Asteroid): number {
    return asteroid.size ** 1.5 * 6;
}

interface AsteroidFragmentData {
    x: number;
    y: number;
    vx: number;
    vy: number;

    angle: number;
    rotationSpeed: number;
    size: number;
    newShapeSeed: number;
}

export function createFragments(asteroid: Asteroid): Asteroid[] {
    const randomNumbers = generateRandomNumbers(asteroid.shapeSeed, 24);
    const randomRotationSpeeds = randomNumbers.slice(0, 3);
    const randomAngles = randomNumbers.slice(3, 6);
    const randomXVels = randomNumbers.slice(6, 9);
    const randomYVels = randomNumbers.slice(9, 12);
    const randomSizes = randomNumbers.slice(12, 15);
    const randomShapeSeeds = randomNumbers.slice(15, 18);
    const randomXPositions = randomNumbers.slice(18, 21);
    const randomYPositions = randomNumbers.slice(21, 24);

    const asteroidData: AsteroidFragmentData[] = [];

    if (asteroid.size > 50) {
        for (let i = 0; i < 3; i++) {
            const angle = randomAngles[i] * Math.PI * 2;
            const rotationSpeed = randomRotationSpeeds[i] * 4 - 2;

            const x = lerp(asteroid.posX - 10, asteroid.posX + 10, randomXPositions[i]);
            const y = lerp(asteroid.posY - 10, asteroid.posY + 10, randomYPositions[i]);

            const vx = asteroid.velX * (randomXVels[i] * 1.5);
            const vy = asteroid.velY * (randomYVels[i] * 1.5);

            const size = lerp(20, 30, randomSizes[i]);
            const newShapeSeed = randomShapeSeeds[i];

            asteroidData.push({
                x,
                y,
                vx,
                vy,
                angle,
                rotationSpeed,
                size,
                newShapeSeed,
            });
        }
    } else if (asteroid.size > 25) {
        for (let i = 0; i < 2; i++) {
            const x = lerp(asteroid.posX - 10, asteroid.posX + 10, randomXPositions[i]);
            const y = lerp(asteroid.posY - 10, asteroid.posY + 10, randomYPositions[i]);

            const vx = asteroid.velX * (randomXVels[i] * 1.5);
            const vy = asteroid.velY * (randomYVels[i] * 1.5);

            const angle = randomAngles[i] * Math.PI * 2;
            const rotationSpeed = lerp(-2, 2, randomRotationSpeeds[i]);

            const size = lerp(15, 20, randomSizes[i]);
            const newShapeSeed = randomShapeSeeds[i];

            asteroidData.push({
                x,
                y,
                vx,
                vy,
                angle,
                rotationSpeed,
                size,
                newShapeSeed,
            });
        }
    }

    const asteroids: Asteroid[] = [];

    for (const fragment of asteroidData) {
        const { x: posX, y: posY, vx: velX, vy: velY } = fragment;
        const { angle, rotationSpeed, size, newShapeSeed } = fragment;

        const newAsteroid: Asteroid = {
            id: nanoid(16),
            size,
            dead: false,
            posX,
            posY,
            velX,
            velY,
            angle,
            rotationSpeed,
            shapeSeed: newShapeSeed,
            collisionInfo: {
                hitBullets: [],
                didHitAsteroid: false,
                didHitPlayer: false,
            },
        };

        asteroids.push(newAsteroid);
    }

    return asteroids;
}

export function cloneAsteroid(asteroid: Asteroid): Asteroid {
    return {
        ...asteroid,
        collisionInfo: {
            ...asteroid.collisionInfo,
        },
    };
}
