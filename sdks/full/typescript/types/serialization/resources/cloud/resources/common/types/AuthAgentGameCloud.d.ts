/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
export declare const AuthAgentGameCloud: core.serialization.ObjectSchema<serializers.cloud.AuthAgentGameCloud.Raw, Rivet.cloud.AuthAgentGameCloud>;
export declare namespace AuthAgentGameCloud {
    interface Raw {
        game_id: string;
    }
}
