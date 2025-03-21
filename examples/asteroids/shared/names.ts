export function sequentialNumberName(taken: Set<string>): string {
    for (let i = 1; true; i++) {
        if (!taken.has(`Player ${i}`)) {
            return `Player ${i}`;
        }
    }
}
