/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as environments from "../../../../environments";
import * as core from "../../../../core";
import { Datacenters } from "../resources/datacenters/client/Client";
import { Servers } from "../resources/servers/client/Client";
export declare namespace Provision {
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
export declare class Provision {
    protected readonly _options: Provision.Options;
    constructor(_options?: Provision.Options);
    protected _datacenters: Datacenters | undefined;
    get datacenters(): Datacenters;
    protected _servers: Servers | undefined;
    get servers(): Servers;
}