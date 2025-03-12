import GameState from "./gamestate";
import Player, { PLAYER_CONSTS } from "./player";
import generateRandomNumbers from "./prng";
import { idToRand, lerp, polar } from "./utils";

export const PARTICLE_TIME = 2;

export interface Particle {
    x: number;
    y: number;
    angle: number;

    vx: number;
    vy: number;
    rotSpeed: number;

    size: number;
}
export default interface ParticleSet {
    playerId: string;
    displayPos: { x: number; y: number };
    displayVel: { x: number; y: number };
    timeLeft: number;
    parts: Particle[];
}



// const tipX = Math.cos(angle) * drawSize;
// const tipY = Math.sin(angle) * drawSize;

// const leftX = Math.cos(angle - Math.PI * 0.8) * drawSize;
// const leftY = Math.sin(angle - Math.PI * 0.8) * drawSize;

// const backX = Math.cos(angle + Math.PI) * drawSize * 0.5;
// const backY = Math.sin(angle + Math.PI) * drawSize * 0.5;

// const rightX = Math.cos(angle + Math.PI * 0.8) * drawSize;
// const rightY = Math.sin(angle + Math.PI * 0.8) * drawSize;

// const fireLeftX = Math.cos(angle - Math.PI * 0.87) * drawSize * 0.87;
// const fireLeftY = Math.sin(angle - Math.PI * 0.87) * drawSize * 0.87;

// const fireBackX = Math.cos(angle + Math.PI) * drawSize * 1.2;
// const fireBackY = Math.sin(angle + Math.PI) * drawSize * 1.2;

// const fireRightX = Math.cos(angle + Math.PI * 0.87) * drawSize * 0.87;
// const fireRightY = Math.sin(angle + Math.PI * 0.87) * drawSize * 0.87;

// return {
//     tip: { x: tipX, y: tipY },
//     left: { x: leftX, y: leftY },
//     back: { x: backX, y: backY },
//     right: { x: rightX, y: rightY },

//     fireLeft: { x: fireLeftX, y: fireLeftY },
//     fireBack: { x: fireBackX, y: fireBackY },
//     fireRight: { x: fireRightX, y: fireRightY },
// };

const PARTICLE_DEFAULTS = [
    {
        angCent: Math.PI * -0.4,
        angPart: Math.PI * 0.1,
        dist: 0.31,
        lenPart: 1.9,
    },
    {
        angCent: Math.PI * 0.4,
        angPart: Math.PI * -0.1,
        dist: 0.31,
        lenPart: 1.9,
    },
    
    {
        angCent: Math.PI * -1.13,
        angPart: Math.PI * -1.34,
        dist: 0.717,
        lenPart: 2/3,
    },
    {
        angCent: Math.PI * 1.13,
        angPart: Math.PI * 1.34,
        dist: 0.717,
        lenPart: 2/3,
    },
];

export function fromPlayer(player: Player, playerIsSelf: boolean): ParticleSet {
    const randomNumbers = generateRandomNumbers(idToRand(player.id), 12);

    const parts: Particle[] = [];

    for (const particleDefault of PARTICLE_DEFAULTS) {
        const [offsetX, offsetY] = polar(
            particleDefault.angCent + player.angle,
            particleDefault.dist * PLAYER_CONSTS.SIZE,
        );
        const [x, y] = [player.posX + offsetX, player.posY + offsetY];

        const angle = particleDefault.angPart + player.angle;
        const size = particleDefault.lenPart * PLAYER_CONSTS.SIZE;

        const color = playerIsSelf ? "#00FFFF" : "#FF0000";
        const vx = lerp(-50, 50, randomNumbers.shift() ?? 0) + player.velX;
        const vy = lerp(-50, 50, randomNumbers.shift() ?? 0) + player.velY;
        const rotSpeed = lerp(-Math.PI, Math.PI, randomNumbers.shift() ?? 0);
        // const vx = 0;
        // const vy = 0;
        // const rotSpeed = 0;

        parts.push({ x, y, angle, size, vx, vy, rotSpeed });
    }

    return {
        playerId: player.id,
        displayPos: { x: player.posX, y: player.posY },
        displayVel: { x: player.velX, y: player.velY },
        timeLeft: PARTICLE_TIME,
        parts,
    };
}

export function cloneParticles(particles: ParticleSet): ParticleSet {
    return {
        playerId: particles.playerId,
        timeLeft: particles.timeLeft,
        displayPos: { ...particles.displayPos },
        displayVel: { ...particles.displayVel },
        parts: particles.parts.map(part => ({ ...part })),
    };
}

export function updateParticles(particles: ParticleSet, game: GameState, dt: number) {

    for (const particle of particles.parts) {
        particle.vx *= 0.1 ** dt;
        particle.vy *= 0.1 ** dt;
        
        particle.x += particle.vx * dt;
        particle.y += particle.vy * dt;
        particle.angle += particle.rotSpeed * dt;
    }

    particles.displayVel.x *= 0.04 ** dt;
    particles.displayVel.y *= 0.04 ** dt;
    particles.displayPos.x += particles.displayVel.x * dt;
    particles.displayPos.y += particles.displayVel.y * dt;

    particles.timeLeft -= dt;
}
