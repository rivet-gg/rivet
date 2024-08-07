/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
import { DisplayName as common$$displayName } from "../../../../../../common/types/DisplayName";
import { PrepareFile as upload_common$$prepareFile } from "../../../../../../upload/resources/common/types/PrepareFile";
import { common, upload } from "../../../../../../index";

export const CreateGameCdnSiteRequest: core.serialization.ObjectSchema<
    serializers.cloud.games.CreateGameCdnSiteRequest.Raw,
    Rivet.cloud.games.CreateGameCdnSiteRequest
> = core.serialization.object({
    displayName: core.serialization.property("display_name", common$$displayName),
    files: core.serialization.list(upload_common$$prepareFile),
});

export declare namespace CreateGameCdnSiteRequest {
    interface Raw {
        display_name: common.DisplayName.Raw;
        files: upload.PrepareFile.Raw[];
    }
}
