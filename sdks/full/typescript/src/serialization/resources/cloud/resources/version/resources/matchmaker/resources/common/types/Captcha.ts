/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../../../index";
import * as Rivet from "../../../../../../../../../../api/index";
import * as core from "../../../../../../../../../../core";
import { CaptchaHcaptcha as cloud_version_matchmaker_common$$captchaHcaptcha } from "./CaptchaHcaptcha";
import { CaptchaTurnstile as cloud_version_matchmaker_common$$captchaTurnstile } from "./CaptchaTurnstile";
import { cloud } from "../../../../../../../../index";

export const Captcha: core.serialization.ObjectSchema<
    serializers.cloud.version.matchmaker.Captcha.Raw,
    Rivet.cloud.version.matchmaker.Captcha
> = core.serialization.object({
    requestsBeforeReverify: core.serialization.property("requests_before_reverify", core.serialization.number()),
    verificationTtl: core.serialization.property("verification_ttl", core.serialization.number()),
    hcaptcha: cloud_version_matchmaker_common$$captchaHcaptcha.optional(),
    turnstile: cloud_version_matchmaker_common$$captchaTurnstile.optional(),
});

export declare namespace Captcha {
    interface Raw {
        requests_before_reverify: number;
        verification_ttl: number;
        hcaptcha?: cloud.version.matchmaker.CaptchaHcaptcha.Raw | null;
        turnstile?: cloud.version.matchmaker.CaptchaTurnstile.Raw | null;
    }
}
