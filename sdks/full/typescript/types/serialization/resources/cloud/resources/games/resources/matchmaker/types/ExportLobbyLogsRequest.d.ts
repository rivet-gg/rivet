/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const ExportLobbyLogsRequest: core.serialization.ObjectSchema<serializers.cloud.games.ExportLobbyLogsRequest.Raw, Rivet.cloud.games.ExportLobbyLogsRequest>;
export declare namespace ExportLobbyLogsRequest {
    interface Raw {
        stream: serializers.cloud.games.LogStream.Raw;
    }
}