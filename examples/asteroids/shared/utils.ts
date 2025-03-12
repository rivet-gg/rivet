export function lerp(a: number, b: number, t: number) {
    return a * (1 - t) + b * t;
}
export function delerp(a: number, b: number, val: number) {
    return (val - a) / (b - a);
}

export const TAU = Math.PI * 2;

export function angZeroToTau(angle: number): number {
    angle %= TAU;
    angle += TAU;
    angle %= TAU;
    return angle;
}

export function angNegPiToPi(angle: number): number {
    angle = angZeroToTau(angle);
    angle += Math.PI;
    angle %= TAU;
    angle -= Math.PI;
    return angle;
}

export function lerpAngle(a: number, b: number, t: number) {
    const diff = angNegPiToPi(b - a);
    return angZeroToTau(a + diff * t);
}

export function clamp(min: number, value: number, max: number) {
    return Math.min(Math.max(min, value), max);
}

export function polarX(angle: number, magnitude: number) {
    return Math.cos(angle) * magnitude;
}
export function polarY(angle: number, magnitude: number) {
    return Math.sin(angle) * magnitude;
}
export function polar(angle: number, magnitude: number) {
    return [polarX(angle, magnitude), polarY(angle, magnitude)] as const;
}
export function polarObj({ angle, rad }: { angle: number; rad: number }) {
    return polar(angle, rad);
}

export function withinRadius(a: { x: number; y: number }, b: { x: number; y: number }, radius: number) {
    const dx = a.x - b.x;
    const dy = a.y - b.y;
    const distSq = dx * dx + dy * dy;
    return distSq < radius ** 2;
}

export function idToRand(id: string): number {
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
        hash = ((hash << 5) - hash) + id.charCodeAt(i);
        hash &= 0xFFFFFFFF;
    }
    return hash / 0x100000000;
}
