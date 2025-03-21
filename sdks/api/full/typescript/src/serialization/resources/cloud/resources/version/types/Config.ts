/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { EngineConfig } from "../resources/engine/types/EngineConfig";
import { CdnConfig } from "../resources/cdn/types/CdnConfig";
import { MatchmakerConfig } from "../resources/matchmaker/types/MatchmakerConfig";
import { KvConfig } from "../resources/kv/types/KvConfig";
import { IdentityConfig } from "../resources/identity/types/IdentityConfig";

export const Config: core.serialization.ObjectSchema<serializers.cloud.version.Config.Raw, Rivet.cloud.version.Config> =
    core.serialization.object({
        scripts: core.serialization.record(core.serialization.string(), core.serialization.string()).optional(),
        engine: EngineConfig.optional(),
        cdn: CdnConfig.optional(),
        matchmaker: MatchmakerConfig.optional(),
        kv: KvConfig.optional(),
        identity: IdentityConfig.optional(),
    });

export declare namespace Config {
    export interface Raw {
        scripts?: Record<string, string> | null;
        engine?: EngineConfig.Raw | null;
        cdn?: CdnConfig.Raw | null;
        matchmaker?: MatchmakerConfig.Raw | null;
        kv?: KvConfig.Raw | null;
        identity?: IdentityConfig.Raw | null;
    }
}
