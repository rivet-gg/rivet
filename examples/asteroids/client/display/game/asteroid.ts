import Asteroid from "@shared/asteroid";

import generateRandomNumbers from "@shared/prng";
import { lerp, polarObj } from "@shared/utils";

export default function drawAsteroid(ctx: CanvasRenderingContext2D, asteroid: Asteroid) {
    const { posX: x, posY: y, angle } = asteroid;

    // Save the state of `ctx` so that it can later be restored
    // (The drawing of the asteroid shouldn't affect anything else)
    ctx.save();

    // This reduces the explicit math to draw the asteroid
    ctx.translate(x, y);
    ctx.rotate(angle);

    // Shape returns points on the asteroid in polar form.
    const shape = asteroidShape(asteroid);

    ctx.strokeStyle = "white";
    ctx.lineWidth = 3;

    ctx.beginPath();

    {
        // By starting by moving to the last point, it guarantees that the
        // asteroid shape will end up being fully closed.
        ctx.moveTo(...polarObj(shape[shape.length - 1]));
    }

    for (const point of shape) {
        ctx.lineTo(...polarObj(point));
    }

    // `ctx.stroke`
    ctx.stroke();
    ctx.restore();
}

const ASTEROID_POINTS = 16;
const RADIANS_PER_POINT = (Math.PI * 2) / ASTEROID_POINTS;

const INSET_FACTOR = -2 / 3;
const OUTSET_FACTOR = 1 / 10;

// FIXME: "outset" is not the correct term.
function asteroidShape(asteroid: Asteroid): { angle: number; rad: number }[] {
    const randomNumbersNeeded = ASTEROID_POINTS * 2 + 1;
    const randomNumbers = generateRandomNumbers(asteroid.shapeSeed, randomNumbersNeeded);

    const randomAngleNumbers = randomNumbers.slice(0, ASTEROID_POINTS);
    const randomRadNumbers = randomNumbers.slice(ASTEROID_POINTS, ASTEROID_POINTS * 2);

    // Put modulus in range 3 - 6
    // Modulus is used to determine how often the asteroid will have an inset vs
    // an outset
    const modulus = lerp(3, 6, randomNumbers[ASTEROID_POINTS * 2]);

    const output: { angle: number; rad: number }[] = [];

    for (let i = 0; i < ASTEROID_POINTS; i++) {
        // Find angle for this point
        const angleFactor = i + randomAngleNumbers[i];
        const angle = angleFactor * RADIANS_PER_POINT;

        // Find radius for this point
        const radRandom = randomRadNumbers[i];
        const isInset = i % modulus < 1;

        // The inset will be used if the modulus matches (isInset is true), the
        // outset will be used otherwise.
        const inset = radRandom * INSET_FACTOR * asteroid.size;
        const outset = radRandom * OUTSET_FACTOR * asteroid.size;

        // Add the correct offset to the radius
        const radOffset = isInset ? inset : outset;
        const rad = asteroid.size * 1.1 + radOffset;

        output.push({ angle, rad });
    }

    return output;
}
