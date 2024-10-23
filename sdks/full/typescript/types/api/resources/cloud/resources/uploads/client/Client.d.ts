/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as environments from "../../../../../../environments";
import * as core from "../../../../../../core";
export declare namespace Uploads {
    interface Options {
        environment?: core.Supplier<environments.RivetEnvironment | string>;
        token?: core.Supplier<core.BearerToken | undefined>;
        fetcher?: core.FetchFunction;
    }
    interface RequestOptions {
        /** The maximum time to wait for a response in seconds. */
        timeoutInSeconds?: number;
        /** The number of times to retry the request. Defaults to 2. */
        maxRetries?: number;
        /** A hook to abort the request. */
        abortSignal?: AbortSignal;
    }
}
export declare class Uploads {
    protected readonly _options: Uploads.Options;
    constructor(_options?: Uploads.Options);
    /**
     * Marks an upload as complete.
     *
     * @param {string} uploadId
     * @param {Uploads.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.cloud.uploads.completeUpload("d5e9c84f-c2b2-4bf4-b4b0-7ffd7a9ffc32")
     */
    completeUpload(uploadId: string, requestOptions?: Uploads.RequestOptions): Promise<void>;
    protected _getAuthorizationHeader(): Promise<string | undefined>;
}