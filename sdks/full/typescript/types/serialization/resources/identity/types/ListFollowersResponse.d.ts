/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { identity, common } from "../../index";
export declare const ListFollowersResponse: core.serialization.ObjectSchema<serializers.identity.ListFollowersResponse.Raw, Rivet.identity.ListFollowersResponse>;
export declare namespace ListFollowersResponse {
    interface Raw {
        identities: identity.Handle.Raw[];
        anchor?: string | null;
        watch: common.WatchResponse.Raw;
    }
}