import GameState, { players } from "./gamestate";
import Player from "./player";

export default interface Score {
    asteroids: number;
    players: number;
}

export function calcScore(score: Score): number {
    return score.asteroids + score.players * 2;
}

export interface ScoreboardEntry {
    player: Player;
    scoreNum: number;
}

export function orderPlayersByScoreDesc(game: GameState): ScoreboardEntry[] {
    const order: ScoreboardEntry[] = [];

    for (const player of players(game)) {
        order.push({ player, scoreNum: calcScore(player.score) });
    }

    order.sort(({ player: { name: a } }, { player: { name: b } }) => a.localeCompare(b));
    order.sort(({ scoreNum: a }, { scoreNum: b }) => b - a);

    return order;
}
