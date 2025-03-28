/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";

export const RegionTier: core.serialization.ObjectSchema<serializers.cloud.RegionTier.Raw, Rivet.cloud.RegionTier> =
    core.serialization.object({
        tierNameId: core.serialization.property("tier_name_id", core.serialization.string()),
        rivetCoresNumerator: core.serialization.property("rivet_cores_numerator", core.serialization.number()),
        rivetCoresDenominator: core.serialization.property("rivet_cores_denominator", core.serialization.number()),
        cpu: core.serialization.number(),
        memory: core.serialization.number(),
        disk: core.serialization.number(),
        bandwidth: core.serialization.number(),
        pricePerSecond: core.serialization.property("price_per_second", core.serialization.number()),
    });

export declare namespace RegionTier {
    export interface Raw {
        tier_name_id: string;
        rivet_cores_numerator: number;
        rivet_cores_denominator: number;
        cpu: number;
        memory: number;
        disk: number;
        bandwidth: number;
        price_per_second: number;
    }
}
