/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const CustomAvatarSummary: core.serialization.ObjectSchema<serializers.cloud.CustomAvatarSummary.Raw, Rivet.cloud.CustomAvatarSummary>;
export declare namespace CustomAvatarSummary {
    interface Raw {
        upload_id: string;
        display_name: string;
        create_ts: string;
        url?: string | null;
        content_length: number;
        complete: boolean;
    }
}