/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Config as cloud_version_engine$$config } from "../resources/engine/types/Config";
import { Config as cloud_version_cdn$$config } from "../resources/cdn/types/Config";
import { Config as cloud_version_matchmaker$$config } from "../resources/matchmaker/types/Config";
import { Config as cloud_version_kv$$config } from "../resources/kv/types/Config";
import { Config as cloud_version_identity$$config } from "../resources/identity/types/Config";
import { cloud } from "../../../../index";

export const Config: core.serialization.ObjectSchema<serializers.cloud.version.Config.Raw, Rivet.cloud.version.Config> =
    core.serialization.object({
        scripts: core.serialization.record(core.serialization.string(), core.serialization.string()).optional(),
        engine: cloud_version_engine$$config.optional(),
        cdn: cloud_version_cdn$$config.optional(),
        matchmaker: cloud_version_matchmaker$$config.optional(),
        kv: cloud_version_kv$$config.optional(),
        identity: cloud_version_identity$$config.optional(),
    });

export declare namespace Config {
    interface Raw {
        scripts?: Record<string, string> | null;
        engine?: cloud.version.engine.Config.Raw | null;
        cdn?: cloud.version.cdn.Config.Raw | null;
        matchmaker?: cloud.version.matchmaker.Config.Raw | null;
        kv?: cloud.version.kv.Config.Raw | null;
        identity?: cloud.version.identity.Config.Raw | null;
    }
}
