/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../..";
import * as Rivet from "../../../../api";
import * as core from "../../../../core";
export declare const PrepareAvatarUploadResponse: core.serialization.ObjectSchema<serializers.group.PrepareAvatarUploadResponse.Raw, Rivet.group.PrepareAvatarUploadResponse>;
export declare namespace PrepareAvatarUploadResponse {
    interface Raw {
        upload_id: string;
        presigned_request: serializers.upload.PresignedRequest.Raw;
    }
}