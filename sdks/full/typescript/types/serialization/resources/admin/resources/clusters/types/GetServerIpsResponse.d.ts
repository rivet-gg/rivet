/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const GetServerIpsResponse: core.serialization.ObjectSchema<serializers.admin.clusters.GetServerIpsResponse.Raw, Rivet.admin.clusters.GetServerIpsResponse>;
export declare namespace GetServerIpsResponse {
    interface Raw {
        ips: string[];
    }
}