/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
export declare const Status: core.serialization.Schema<serializers.identity.Status.Raw, Rivet.identity.Status>;
export declare namespace Status {
    type Raw = "online" | "away" | "offline";
}
