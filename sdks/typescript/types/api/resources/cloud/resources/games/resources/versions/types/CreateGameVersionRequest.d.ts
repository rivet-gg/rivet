/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as Rivet from "../../../../../../..";
export interface CreateGameVersionRequest {
    /** Represent a resource's readable display name. */
    displayName: string;
    config: Rivet.cloud.version.Config;
}