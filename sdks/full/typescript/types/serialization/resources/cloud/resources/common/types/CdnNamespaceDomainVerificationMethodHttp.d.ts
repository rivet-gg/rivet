/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
export declare const CdnNamespaceDomainVerificationMethodHttp: core.serialization.ObjectSchema<serializers.cloud.CdnNamespaceDomainVerificationMethodHttp.Raw, Rivet.cloud.CdnNamespaceDomainVerificationMethodHttp>;
export declare namespace CdnNamespaceDomainVerificationMethodHttp {
    interface Raw {
        cname_record: string;
    }
}
