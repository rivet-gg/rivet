/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
export declare const AddNamespaceDomainRequest: core.serialization.ObjectSchema<serializers.cloud.games.namespaces.AddNamespaceDomainRequest.Raw, Rivet.cloud.games.namespaces.AddNamespaceDomainRequest>;
export declare namespace AddNamespaceDomainRequest {
    interface Raw {
        domain: string;
    }
}
