/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const GetDeviceLinkResponse: core.serialization.ObjectSchema<serializers.cloud.devices.GetDeviceLinkResponse.Raw, Rivet.cloud.devices.GetDeviceLinkResponse>;
export declare namespace GetDeviceLinkResponse {
    interface Raw {
        cloud_token?: string | null;
        watch: serializers.WatchResponse.Raw;
    }
}