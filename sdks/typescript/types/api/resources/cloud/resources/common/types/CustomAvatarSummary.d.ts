/**
 * This file was auto-generated by Fern from our API Definition.
 */
/**
 * A custom avatar summary.
 */
export interface CustomAvatarSummary {
    uploadId: string;
    /** Represent a resource's readable display name. */
    displayName: string;
    /** RFC3339 timestamp. */
    createTs: Date;
    /** The URL of this custom avatar image. Only present if upload is complete. */
    url?: string;
    /** Unsigned 64 bit integer. */
    contentLength: number;
    /** Whether or not this custom avatar has completely been uploaded. */
    complete: boolean;
}