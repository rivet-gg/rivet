/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const LogsLobbyStatusStopped: core.serialization.ObjectSchema<serializers.cloud.LogsLobbyStatusStopped.Raw, Rivet.cloud.LogsLobbyStatusStopped>;
export declare namespace LogsLobbyStatusStopped {
    interface Raw {
        stop_ts: string;
        failed: boolean;
        exit_code: number;
    }
}