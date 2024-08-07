/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
import { common } from "../../../../../../index";
export declare const GetDeviceLinkResponse: core.serialization.ObjectSchema<serializers.cloud.devices.GetDeviceLinkResponse.Raw, Rivet.cloud.devices.GetDeviceLinkResponse>;
export declare namespace GetDeviceLinkResponse {
    interface Raw {
        cloud_token?: string | null;
        watch: common.WatchResponse.Raw;
    }
}
