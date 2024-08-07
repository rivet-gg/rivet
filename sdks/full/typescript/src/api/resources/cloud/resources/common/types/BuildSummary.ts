/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../../index";

/**
 * A build summary.
 */
export interface BuildSummary {
    buildId: string;
    uploadId: string;
    displayName: Rivet.DisplayName;
    createTs: Rivet.Timestamp;
    /** Unsigned 64 bit integer. */
    contentLength: number;
    /** Whether or not this build has completely been uploaded. */
    complete: boolean;
    /** Tags of this build */
    tags: Record<string, string>;
}
