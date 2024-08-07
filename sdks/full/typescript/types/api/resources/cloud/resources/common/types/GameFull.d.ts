/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as Rivet from "../../../../../index";
/**
 * A full game.
 */
export interface GameFull {
    gameId: string;
    createTs: Rivet.Timestamp;
    /** A human readable short identifier used to references resources. Different than a `rivet.common#Uuid` because this is intended to be human readable. Different than `rivet.common#DisplayName` because this should not include special characters and be short. */
    nameId: string;
    displayName: Rivet.DisplayName;
    developerGroupId: string;
    /** Unsigned 32 bit integer. */
    totalPlayerCount: number;
    /** The URL of this game's logo image. */
    logoUrl?: string;
    /** The URL of this game's banner image. */
    bannerUrl?: string;
    /** A list of namespace summaries. */
    namespaces: Rivet.cloud.NamespaceSummary[];
    /** A list of version summaries. */
    versions: Rivet.cloud.version.Summary[];
    /** A list of region summaries. */
    availableRegions: Rivet.cloud.RegionSummary[];
}
