/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";

export const CdnAuthType: core.serialization.Schema<serializers.cloud.CdnAuthType.Raw, Rivet.cloud.CdnAuthType> =
    core.serialization.enum_(["none", "basic"]);

export declare namespace CdnAuthType {
    export type Raw = "none" | "basic";
}
