/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Jwt } from "../../../../common/types/Jwt";

export const JoinPlayer: core.serialization.ObjectSchema<
    serializers.matchmaker.JoinPlayer.Raw,
    Rivet.matchmaker.JoinPlayer
> = core.serialization.object({
    token: Jwt,
});

export declare namespace JoinPlayer {
    export interface Raw {
        token: Jwt.Raw;
    }
}
