import Asteroid, { createFragments } from "./asteroid";
import Bullet from "./bullet";
import Player, { PLAYER_CONSTS } from "./player";
import ParticleSet, { fromPlayer } from "./particles";

export default interface GameState {
    asteroids: Record<string, Asteroid>;
    bullets: Record<string, Bullet>;
    players: Record<string, Player>;
    particleSets: Record<string, ParticleSet>,

    size: { w: number; h: number };

    physicsTime: number;
    targetAsteroids: number;
}

import { preUpdateAsteroid, updateAsteroid } from "./asteroid";
import { preUpdateBullet, updateBullet } from "./bullet";
import { preUpdatePlayer, updatePlayer } from "./player";
import { updateParticles } from "./particles";


export function updateGame(game: GameState, dt: number, thisPlayerId: string) {
    dt = Math.min(dt, 0.1);
    for (const asteroid of asteroids(game)) preUpdateAsteroid(asteroid, game, dt);
    for (const bullet of bullets(game)) preUpdateBullet(bullet, game, dt);
    for (const player of players(game)) preUpdatePlayer(player, game, dt);

    for (const asteroid of asteroids(game)) updateAsteroid(asteroid, game, dt);
    for (const bullet of bullets(game)) updateBullet(bullet, game, dt);
    for (const player of players(game)) updatePlayer(player, game, dt);
    for (const particleSet of particleSets(game)) updateParticles(particleSet, game, dt);

    for (const asteroid of asteroids(game)) {
        if (asteroid.dead) {
            const newAsteroids = createFragments(asteroid);
            for (const fragment of newAsteroids) game.asteroids[fragment.id] = fragment;
            delete game.asteroids[asteroid.id];
        }
    }
    for (const bullet of bullets(game)) if (bullet.dead) delete game.bullets[bullet.id];
    for (const player of players(game)) if (player.dead && player.id) {
        const particles = fromPlayer(player, player.id === thisPlayerId);
        game.particleSets[particles.playerId] = particles;
        if (player.id !== thisPlayerId) delete game.players[player.id];
    }    
    for (const particleSet of particleSets(game)) {
        if (particleSet.playerId === thisPlayerId) continue;
        if (particleSet.timeLeft <= 0) delete game.particleSets[particleSet.playerId];
    }

    game.physicsTime += dt;
}

export function ensureAsteroidCount(game: GameState): void {
    const asteroidDeficit = game.targetAsteroids - Object.keys(game.asteroids).length;
    if (asteroidDeficit <= 0) return;

    const asteroidList = asteroids(game);

    for (let i = 0; i < asteroidDeficit; i++) {
        const randomAsteroid = asteroidList[Math.floor(Math.random() * asteroidList.length)];

        if (randomAsteroid) {
            const newAsteroid = newRandomAsteroid(
                [25, 60],
                [randomAsteroid.posX, randomAsteroid.posX],
                [randomAsteroid.posY, randomAsteroid.posY],
                100,
                [-2, 2],
            );
    
            game.asteroids[newAsteroid.id] = newAsteroid;
        } else {
            const spawnpoint = getClearSpawnpoint(game, 80, 100);
            const newAsteroid = newRandomAsteroid(
                [25, 60],
                [spawnpoint.x, spawnpoint.x],
                [spawnpoint.y, spawnpoint.y],
                100,
                [-2, 2],
            );
    
            game.asteroids[newAsteroid.id] = newAsteroid;
        }

    }
}


import { newRandomAsteroid } from "./asteroid";

export function newRandomGame(mapSize: { x: number; y: number }, asteroids: number = 45): GameState {
    const outputState: GameState = {
        asteroids: {},
        bullets: {},
        players: {},
        particleSets: {},

        size: { w: mapSize.x, h: mapSize.y },

        physicsTime: 0,
        targetAsteroids: asteroids,
    };

    const { x: w, y: h } = mapSize;
    for (let i = 0; i < asteroids; i++) {
        const newAsteroid = newRandomAsteroid([25, 60], [0, w], [0, h], 100, [-2, 2]);
        outputState.asteroids[newAsteroid.id] = newAsteroid;
    }

    return outputState;
}

export function newBulletHellGame(mapSize: { x: number; y: number }, asteroids: number = 200): GameState {
    const outputState: GameState = {
        asteroids: {},
        bullets: {},
        players: {},
        particleSets: {},

        size: { w: mapSize.x, h: mapSize.y },

        physicsTime: 0,
        targetAsteroids: 0,
    };

    const { x: w, y: h } = mapSize;
    for (let i = 0; i < asteroids; i++) {
        const newAsteroid = newRandomAsteroid([10, 25], [0, w], [0, h], 100, [-2, 2]);
        outputState.asteroids[newAsteroid.id] = newAsteroid;
    }

    return outputState;
}

import { cloneAsteroid } from "./asteroid";
import { cloneBullet } from "./bullet";
import { clonePlayer } from "./player";
import { cloneParticles } from "./particles";

export function cloneGameState(state: GameState, deep: boolean): GameState {
    const asteroidEntries = Object.entries(state.asteroids);
    const clonedAsteroidEntries: [string, Asteroid][] = asteroidEntries.map(([id, asteroid]) => [
        id,
        deep ? cloneAsteroid(asteroid) : asteroid,
    ]);
    const asteroids = Object.fromEntries(clonedAsteroidEntries);

    const bulletEntries = Object.entries(state.bullets);
    const clonedBulletEntries: [string, Bullet][] = bulletEntries.map(([id, bullet]) => [
        id,
        deep ? cloneBullet(bullet) : bullet,
    ]);
    const bullets = Object.fromEntries(clonedBulletEntries);

    const playerEntries = Object.entries(state.players);
    const clonedPlayerEntries: [string, Player][] = playerEntries.map(([id, player]) => [
        id,
        deep ? clonePlayer(player) : player,
    ]);
    const players = Object.fromEntries(clonedPlayerEntries);

    const particleEntries = Object.entries(state.particleSets);
    const clonedParticleEntries: [string, ParticleSet][] = particleEntries.map(([id, particles]) => [
        id,
        deep ? cloneParticles(particles) : particles,
    ]);
    const playerParticles = Object.fromEntries(clonedParticleEntries);

    return {
        asteroids,
        bullets,
        players,
        particleSets: playerParticles,
        size: { ...state.size },
        physicsTime: state.physicsTime,
        targetAsteroids: state.targetAsteroids,
    };
}

export function asteroids(game: GameState): Asteroid[] {
    return Object.values(game.asteroids);
}
export function bullets(game: GameState): Bullet[] {
    return Object.values(game.bullets);
}
export function players(game: GameState): Player[] {
    return Object.values(game.players);
}
export function particleSets(game: GameState): ParticleSet[] {
    return Object.values(game.particleSets);
}

export function getAsteroid(game: GameState, id: string): Asteroid | undefined {
    return game.asteroids[id];
}
export function getBullet(game: GameState, id: string): Bullet | undefined {
    return game.bullets[id];
}
export function getPlayer(game: GameState, id: string): Player | undefined {
    return game.players[id];
}
export function getPlayerParticles(game: GameState, id: string): ParticleSet | undefined {
    return game.particleSets[id];
}

export function getObjectsFromIds(
    game: GameState,
    asteroidIds: string[],
    bulletIds: string[],
    playerIds: string[],
): { asteroids: Asteroid[]; bullets: Bullet[]; players: Player[] } {
    const dedupAsteroidIds = [...new Set(asteroidIds)];
    const dedupBulletIds = [...new Set(bulletIds)];
    const dedupPlayerIds = [...new Set(playerIds)];

    const asteroids = [] as Asteroid[];
    const bullets = [] as Bullet[];
    const players = [] as Player[];

    for (const id of dedupAsteroidIds) {
        const asteroid = game.asteroids[id];
        if (asteroid) asteroids.push(asteroid);
    }
    for (const id of dedupBulletIds) {
        const bullet = game.bullets[id];
        if (bullet) bullets.push(bullet);
    }
    for (const id of dedupPlayerIds) {
        const player = game.players[id];
        if (player) players.push(player);
    }

    return { asteroids, bullets, players };
}

export function takenNames(state: GameState): Set<string> {
    const names = new Set<string>();
    for (const playerId in state.players) {
        names.add(state.players[playerId].name);
    }
    return names;
}

export function canWinGame(game: GameState): boolean {
    return game.targetAsteroids === 0;
}
export function wonGame(game: GameState): boolean {
    return canWinGame(game) && asteroids(game).length === 0;
}
export function isCoOp(game: GameState): boolean {
    return canWinGame(game);
}

export function getClearSpawnpoint(game: GameState, radius: number, tries: number = 25) {
    const { w, h } = game.size;

    const xMin = radius;
    const xMax = w - radius;
    const xRange = xMax - xMin;

    const yMin = radius;
    const yMax = h - radius;
    const yRange = yMax - yMin;

    let x = xMin + Math.random() * xRange;
    let y = yMin + Math.random() * yRange;

    for (let i = 0; i < tries; i++) {
        let collision = false;
        for (const player of players(game)) {
            const diffX = player.posX - x;
            const diffY = player.posY - y;
            const minDist = radius + PLAYER_CONSTS.SIZE;

            if (diffX ** 2 + diffY ** 2 < minDist ** 2) {
                collision = true;
                break;
            }
        }
        for (const asteroid of asteroids(game)) {
            const diffX = asteroid.posX - x;
            const diffY = asteroid.posY - y;
            const minDist = radius + asteroid.size;

            if (diffX ** 2 + diffY ** 2 < minDist ** 2) {
                collision = true;
                break;
            }
        }
        if (!collision) return { x, y };
    }
    console.log("Failed to find clear spawnpoint");
    return { x, y };
}
