/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { StatConfig as game_common$$statConfig } from "./StatConfig";
import { game } from "../../../../index";

export const Stat: core.serialization.ObjectSchema<serializers.game.Stat.Raw, Rivet.game.Stat> =
    core.serialization.object({
        config: game_common$$statConfig,
        overallValue: core.serialization.property("overall_value", core.serialization.number()),
    });

export declare namespace Stat {
    interface Raw {
        config: game.StatConfig.Raw;
        overall_value: number;
    }
}
