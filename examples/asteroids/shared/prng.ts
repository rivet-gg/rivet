// This is a "Linear Congruential Generator" RNG. This allows us to get back the
// same numbers with the same inputs, but still have those numbers seem "random"
// enough.

// These numbers were chosen from
// https://www.ams.org/journals/mcom/1999-68-225/S0025-5718-99-00996-5/S0025-5718-99-00996-5.pdf.
// They're chosen specifically so we don't have to worry about exceeding
// Number.MAX_SAFE_INTEGER, without sacrificing the period of the generator.
//
// I don't understand the math, but I do trust the mathematicians.

const M = 2 ** 28 - 57;
const A = 31792125;

const MASK = 0xffffff;

export function newSeed(): number {
    return Math.floor(Math.random() * M);
}

export default function generateRandomNumbers(seed: number, count: number): number[] {
    let curr = seed;

    const output: number[] = [];

    for (let i = 0; i < count; i++) {
        curr = (curr * A) % M;

        output.push((curr & MASK) / (MASK + 1));
    }

    return output;
}
