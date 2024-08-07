/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common, cloud } from "../../../../index";
export declare const Full: core.serialization.ObjectSchema<serializers.cloud.version.Full.Raw, Rivet.cloud.version.Full>;
export declare namespace Full {
    interface Raw {
        version_id: string;
        create_ts: common.Timestamp.Raw;
        display_name: common.DisplayName.Raw;
        config: cloud.version.Config.Raw;
    }
}
