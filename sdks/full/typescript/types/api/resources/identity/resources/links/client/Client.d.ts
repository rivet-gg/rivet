/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as environments from "../../../../../../environments";
import * as core from "../../../../../../core";
import * as Rivet from "../../../../../index";
export declare namespace Links {
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
export declare class Links {
    protected readonly _options: Links.Options;
    constructor(_options?: Links.Options);
    /**
     * Begins the process for linking an identity with the Rivet Hub.
     *
     * # Importance of Linking Identities
     *
     * When an identity is created via `rivet.api.identity#SetupIdentity`, the identity is temporary
     * and is not shared with other games the user plays.
     * In order to make the identity permanent and synchronize the identity with
     * other games, the identity must be linked with the hub.
     *
     * # Linking Process
     *
     * The linking process works by opening `identity_link_url` in a browser then polling
     * `rivet.api.identity#GetGameLink` to wait for it to complete.
     * This is designed to be as flexible as possible so `identity_link_url` can be opened
     * on any device. For example, when playing a console game, the user can scan a
     * QR code for `identity_link_url` to authenticate on their phone.
     *
     * @param {Links.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.identity.links.prepare()
     */
    prepare(requestOptions?: Links.RequestOptions): Promise<Rivet.identity.PrepareGameLinkResponse>;
    /**
     * Returns the current status of a linking process. Once `status` is `complete`, the identity's profile should be fetched again since they may have switched accounts.
     *
     * @param {Rivet.identity.GetGameLinkRequest} request
     * @param {Links.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.identity.links.get({
     *         identityLinkToken: "string",
     *         watchIndex: "string"
     *     })
     */
    get(request: Rivet.identity.GetGameLinkRequest, requestOptions?: Links.RequestOptions): Promise<Rivet.identity.GetGameLinkResponse>;
    /**
     * Completes a game link process and returns whether or not the link is valid.
     *
     * @param {Rivet.identity.CompleteGameLinkRequest} request
     * @param {Links.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.identity.links.complete({
     *         identityLinkToken: "string"
     *     })
     */
    complete(request: Rivet.identity.CompleteGameLinkRequest, requestOptions?: Links.RequestOptions): Promise<void>;
    /**
     * Cancels a game link. It can no longer be used to link after cancellation.
     *
     * @param {Rivet.identity.CancelGameLinkRequest} request
     * @param {Links.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.identity.links.cancel({
     *         identityLinkToken: "string"
     *     })
     */
    cancel(request: Rivet.identity.CancelGameLinkRequest, requestOptions?: Links.RequestOptions): Promise<void>;
    protected _getAuthorizationHeader(): Promise<string | undefined>;
}