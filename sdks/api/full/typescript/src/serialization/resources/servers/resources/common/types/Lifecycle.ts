/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";

export const Lifecycle: core.serialization.ObjectSchema<serializers.servers.Lifecycle.Raw, Rivet.servers.Lifecycle> =
    core.serialization.object({
        killTimeout: core.serialization.property("kill_timeout", core.serialization.number().optional()),
    });

export declare namespace Lifecycle {
    export interface Raw {
        kill_timeout?: number | null;
    }
}
