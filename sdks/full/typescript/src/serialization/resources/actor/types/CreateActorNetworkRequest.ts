/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { NetworkMode as actor_common$$networkMode } from "../resources/common/types/NetworkMode";
import { CreateActorPortRequest as actor$$createActorPortRequest } from "./CreateActorPortRequest";
import { actor } from "../../index";

export const CreateActorNetworkRequest: core.serialization.ObjectSchema<
    serializers.actor.CreateActorNetworkRequest.Raw,
    Rivet.actor.CreateActorNetworkRequest
> = core.serialization.object({
    mode: actor_common$$networkMode.optional(),
    ports: core.serialization.record(core.serialization.string(), actor$$createActorPortRequest),
});

export declare namespace CreateActorNetworkRequest {
    interface Raw {
        mode?: actor.NetworkMode.Raw | null;
        ports: Record<string, actor.CreateActorPortRequest.Raw>;
    }
}