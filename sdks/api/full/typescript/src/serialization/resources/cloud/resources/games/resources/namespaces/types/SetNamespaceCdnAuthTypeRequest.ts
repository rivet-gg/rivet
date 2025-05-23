/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
import { CdnAuthType } from "../../../../common/types/CdnAuthType";

export const SetNamespaceCdnAuthTypeRequest: core.serialization.ObjectSchema<
    serializers.cloud.games.namespaces.SetNamespaceCdnAuthTypeRequest.Raw,
    Rivet.cloud.games.namespaces.SetNamespaceCdnAuthTypeRequest
> = core.serialization.object({
    authType: core.serialization.property("auth_type", CdnAuthType),
});

export declare namespace SetNamespaceCdnAuthTypeRequest {
    export interface Raw {
        auth_type: CdnAuthType.Raw;
    }
}
