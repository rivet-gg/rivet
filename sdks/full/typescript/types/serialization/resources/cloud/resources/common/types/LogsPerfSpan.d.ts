/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common } from "../../../../index";
export declare const LogsPerfSpan: core.serialization.ObjectSchema<serializers.cloud.LogsPerfSpan.Raw, Rivet.cloud.LogsPerfSpan>;
export declare namespace LogsPerfSpan {
    interface Raw {
        label: string;
        start_ts: common.Timestamp.Raw;
        finish_ts?: common.Timestamp.Raw | null;
        req_id?: string | null;
    }
}