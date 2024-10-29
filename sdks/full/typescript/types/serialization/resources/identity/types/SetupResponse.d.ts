/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { common, identity } from "../../index";
export declare const SetupResponse: core.serialization.ObjectSchema<serializers.identity.SetupResponse.Raw, Rivet.identity.SetupResponse>;
export declare namespace SetupResponse {
    interface Raw {
        identity_token: common.Jwt.Raw;
        identity_token_expire_ts: common.Timestamp.Raw;
        identity: identity.Profile.Raw;
        game_id: string;
    }
}