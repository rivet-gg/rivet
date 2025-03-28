/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../../../index";
import * as Rivet from "../../../../../../../../../../api/index";
import * as core from "../../../../../../../../../../core";

export const CaptchaTurnstile: core.serialization.ObjectSchema<
    serializers.cloud.version.matchmaker.CaptchaTurnstile.Raw,
    Rivet.cloud.version.matchmaker.CaptchaTurnstile
> = core.serialization.object({
    siteKey: core.serialization.property("site_key", core.serialization.string()),
    secretKey: core.serialization.property("secret_key", core.serialization.string()),
});

export declare namespace CaptchaTurnstile {
    export interface Raw {
        site_key: string;
        secret_key: string;
    }
}
