import { cloneParticles } from "@shared/particles";
import { cloneAsteroid } from "../shared/asteroid";
import { cloneBullet } from "../shared/bullet";
import GameState, { asteroids, bullets, cloneGameState, players, updateGame } from "../shared/gamestate";
import { applyPlayerInput, clonePlayer } from "../shared/player";
import { clamp, lerp, lerpAngle } from "../shared/utils";

export default interface ClientGameState {
    serverGameState: GameState;
    clientGameState: GameState;
    serverSync: number;

    running: boolean;
}

export function initClientGamestate(state: GameState, now: number): ClientGameState {
    return {
        serverGameState: state,
        clientGameState: cloneGameState(state, true),
        serverSync: now,
        running: false,
    };
}

function removeDuplicates(clientState: ClientGameState, playerId: string | null) {
    const serverGame = clientState.serverGameState;
    const clientGame = clientState.clientGameState;

    const serverAsteroids = serverGame.asteroids;
    const clientAsteroids = clientGame.asteroids;

    const serverBullets = serverGame.bullets;
    const clientBullets = clientGame.bullets;

    const serverPlayers = serverGame.players;
    const clientPlayers = clientGame.players;

    const serverParticles = serverGame.particleSets;
    const clientParticles = clientGame.particleSets;

    for (const asteroid in serverAsteroids) {
        if (!clientAsteroids[asteroid]) {
            clientAsteroids[asteroid] = cloneAsteroid(serverAsteroids[asteroid]);
        }
    }
    for (const bullet in serverBullets) {
        if (!clientBullets[bullet]) {
            clientBullets[bullet] = cloneBullet(serverBullets[bullet]);
        }
    }
    for (const player in serverPlayers) {
        if (!clientPlayers[player]) {
            clientPlayers[player] = clonePlayer(serverPlayers[player]);
        }
    }
    for (const particleSet in serverParticles) {
        if (!clientParticles[particleSet]) {
            clientParticles[particleSet] = cloneParticles(serverParticles[particleSet]);
        }
    }


    for (const asteroid in clientAsteroids) {
        if (!serverAsteroids[asteroid]) {
            delete clientAsteroids[asteroid];
        }
    }
    for (const bullet in clientBullets) {
        if (!serverBullets[bullet]) {
            delete clientBullets[bullet];
        }
    }
    for (const player in clientPlayers) {
        if (player === playerId) continue;

        if (!serverPlayers[player]) {
            delete clientPlayers[player];
        }
    }
    for (const particleSet in clientParticles) {
        if (!serverParticles[particleSet]) {
            delete clientParticles[particleSet];
        }
    }
}

export function serverSync(
    clientState: ClientGameState,
    serverGame: GameState,
    playerId: string | null,
    now: number,
): void {
    clientState.serverGameState = serverGame;
    clientState.serverSync = now;
    clientState.clientGameState.physicsTime = serverGame.physicsTime;

    removeDuplicates(clientState, playerId);
}

const interpTimeMs = 500;
function interpolate(clientState: ClientGameState, lerpAmount: number) {
    for (const asteroid of asteroids(clientState.clientGameState)) {
        const serverAsteroid = clientState.serverGameState.asteroids[asteroid.id];
        if (!serverAsteroid) continue;

        asteroid.posX = lerp(asteroid.posX, serverAsteroid.posX, lerpAmount);
        asteroid.posY = lerp(asteroid.posY, serverAsteroid.posY, lerpAmount);
        asteroid.velX = lerp(asteroid.velX, serverAsteroid.velX, lerpAmount);
        asteroid.velY = lerp(asteroid.velY, serverAsteroid.velY, lerpAmount);

        asteroid.angle = lerpAngle(asteroid.angle, serverAsteroid.angle, lerpAmount);
        asteroid.rotationSpeed = lerp(asteroid.rotationSpeed, serverAsteroid.rotationSpeed, lerpAmount);

        asteroid.dead = serverAsteroid.dead;
    }
    for (const bullet of bullets(clientState.clientGameState)) {
        const serverBullet = clientState.serverGameState.bullets[bullet.id];
        if (!serverBullet) continue;

        bullet.posX = lerp(bullet.posX, serverBullet.posX, lerpAmount);
        bullet.posY = lerp(bullet.posY, serverBullet.posY, lerpAmount);
        bullet.velX = lerp(bullet.velX, serverBullet.velX, lerpAmount);
        bullet.velY = lerp(bullet.velY, serverBullet.velY, lerpAmount);
    }
    for (const player of players(clientState.clientGameState)) {
        const serverPlayer = clientState.serverGameState.players[player.id];
        if (!serverPlayer) continue;

        player.posX = lerp(player.posX, serverPlayer.posX, lerpAmount);
        player.posY = lerp(player.posY, serverPlayer.posY, lerpAmount);
        player.velX = lerp(player.velX, serverPlayer.velX, lerpAmount);
        player.velY = lerp(player.velY, serverPlayer.velY, lerpAmount);

        player.angle = lerpAngle(player.angle, serverPlayer.angle, lerpAmount);

        player.id = serverPlayer.id;
        player.name = serverPlayer.name;

        player.dead = serverPlayer.dead;
        player.invincibilityTimeLeft = serverPlayer.invincibilityTimeLeft;
        player.playerInput = { ...serverPlayer.playerInput };
        player.score = { ...serverPlayer.score };
        player.lastShot = serverPlayer.lastShot;
    }
}

export function update(clientState: ClientGameState, playerId: string, elapsed: number, now: number): void {
    removeDuplicates(clientState, playerId);

    const prevLerpVar = clamp(0, (now - elapsed - clientState.serverSync) / interpTimeMs, 0.99);
    const lerpVar = clamp(0, (now - clientState.serverSync) / interpTimeMs, 1);
    const lerpAmount = clamp(0, (lerpVar - prevLerpVar) / (1 - prevLerpVar), 1);
    interpolate(clientState, lerpAmount);

    for (const player of players(clientState.clientGameState)) applyPlayerInput(player, elapsed / 1000);
    for (const player of players(clientState.serverGameState)) applyPlayerInput(player, elapsed / 1000);

    updateGame(clientState.clientGameState, elapsed / 1000, playerId);
    updateGame(clientState.serverGameState, elapsed / 1000, playerId);
}
