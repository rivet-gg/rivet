/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";

export const CreateGameNamespaceTokenPublicResponse: core.serialization.ObjectSchema<
    serializers.cloud.games.namespaces.CreateGameNamespaceTokenPublicResponse.Raw,
    Rivet.cloud.games.namespaces.CreateGameNamespaceTokenPublicResponse
> = core.serialization.object({
    token: core.serialization.string(),
});

export declare namespace CreateGameNamespaceTokenPublicResponse {
    export interface Raw {
        token: string;
    }
}
