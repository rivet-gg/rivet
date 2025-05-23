/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../../index";

export interface GameSummary {
    gameId: string;
    nameId: Rivet.Identifier;
    displayName: Rivet.DisplayName;
    /** The URL of this game's logo image. */
    logoUrl?: string;
    /** The URL of this game's banner image. */
    bannerUrl?: string;
    url: string;
    developer: Rivet.group.Handle;
    /** Unsigned 32 bit integer. */
    totalPlayerCount: number;
}
