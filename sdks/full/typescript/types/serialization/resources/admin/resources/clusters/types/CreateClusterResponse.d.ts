/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
export declare const CreateClusterResponse: core.serialization.ObjectSchema<serializers.admin.clusters.CreateClusterResponse.Raw, Rivet.admin.clusters.CreateClusterResponse>;
export declare namespace CreateClusterResponse {
    interface Raw {
        cluster_id: string;
    }
}