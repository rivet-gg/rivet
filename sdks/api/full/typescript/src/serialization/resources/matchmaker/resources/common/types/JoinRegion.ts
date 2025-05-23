/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Identifier } from "../../../../common/types/Identifier";
import { DisplayName } from "../../../../common/types/DisplayName";

export const JoinRegion: core.serialization.ObjectSchema<
    serializers.matchmaker.JoinRegion.Raw,
    Rivet.matchmaker.JoinRegion
> = core.serialization.object({
    regionId: core.serialization.property("region_id", Identifier),
    displayName: core.serialization.property("display_name", DisplayName),
});

export declare namespace JoinRegion {
    export interface Raw {
        region_id: Identifier.Raw;
        display_name: DisplayName.Raw;
    }
}
