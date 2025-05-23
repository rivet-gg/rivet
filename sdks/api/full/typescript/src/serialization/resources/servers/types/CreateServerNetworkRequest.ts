/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { NetworkMode } from "../resources/common/types/NetworkMode";
import { CreateServerPortRequest } from "./CreateServerPortRequest";

export const CreateServerNetworkRequest: core.serialization.ObjectSchema<
    serializers.servers.CreateServerNetworkRequest.Raw,
    Rivet.servers.CreateServerNetworkRequest
> = core.serialization.object({
    mode: NetworkMode.optional(),
    ports: core.serialization.record(core.serialization.string(), CreateServerPortRequest),
});

export declare namespace CreateServerNetworkRequest {
    export interface Raw {
        mode?: NetworkMode.Raw | null;
        ports: Record<string, CreateServerPortRequest.Raw>;
    }
}
