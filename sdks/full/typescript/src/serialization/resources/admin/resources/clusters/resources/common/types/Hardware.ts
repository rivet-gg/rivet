/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";

export const Hardware: core.serialization.ObjectSchema<
    serializers.admin.clusters.Hardware.Raw,
    Rivet.admin.clusters.Hardware
> = core.serialization.object({
    providerHardware: core.serialization.property("provider_hardware", core.serialization.string()),
});

export declare namespace Hardware {
    interface Raw {
        provider_hardware: string;
    }
}