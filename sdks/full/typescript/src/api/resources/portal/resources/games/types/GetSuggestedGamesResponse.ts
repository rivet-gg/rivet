/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../..";

export interface GetSuggestedGamesResponse {
    /** A list of game summaries. */
    games: Rivet.game.Summary[];
    watch: Rivet.WatchResponse;
}