/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const PlatformLink: core.serialization.ObjectSchema<serializers.game.PlatformLink.Raw, Rivet.game.PlatformLink>;
export declare namespace PlatformLink {
    interface Raw {
        display_name: serializers.DisplayName.Raw;
        url: string;
    }
}