/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
import { cloud } from "../../../../../../index";
export declare const ListGameCustomAvatarsResponse: core.serialization.ObjectSchema<serializers.cloud.games.ListGameCustomAvatarsResponse.Raw, Rivet.cloud.games.ListGameCustomAvatarsResponse>;
export declare namespace ListGameCustomAvatarsResponse {
    interface Raw {
        custom_avatars: cloud.CustomAvatarSummary.Raw[];
    }
}
