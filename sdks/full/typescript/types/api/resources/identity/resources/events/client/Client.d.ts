/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as environments from "../../../../../../environments";
import * as core from "../../../../../../core";
import * as Rivet from "../../../../..";
export declare namespace Events {
    interface Options {
        environment?: core.Supplier<environments.RivetEnvironment | string>;
        token?: core.Supplier<core.BearerToken | undefined>;
        fetcher?: core.FetchFunction;
    }
    interface RequestOptions {
        timeoutInSeconds?: number;
        maxRetries?: number;
    }
}
export declare class Events {
    protected readonly _options: Events.Options;
    constructor(_options?: Events.Options);
    /**
     * Returns all events relative to the current identity.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    watch(request?: Rivet.identity.WatchEventsRequest, requestOptions?: Events.RequestOptions): Promise<Rivet.identity.WatchEventsResponse>;
    protected _getAuthorizationHeader(): Promise<string | undefined>;
}