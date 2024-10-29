/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common, matchmaker } from "../../../../index";
export declare const GameModeStatistics: core.serialization.ObjectSchema<serializers.matchmaker.GameModeStatistics.Raw, Rivet.matchmaker.GameModeStatistics>;
export declare namespace GameModeStatistics {
    interface Raw {
        player_count: number;
        regions: Record<common.Identifier.Raw, matchmaker.RegionStatistics.Raw>;
    }
}