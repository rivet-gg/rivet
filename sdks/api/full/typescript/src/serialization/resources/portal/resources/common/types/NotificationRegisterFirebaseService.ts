/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";

export const NotificationRegisterFirebaseService: core.serialization.ObjectSchema<
    serializers.portal.NotificationRegisterFirebaseService.Raw,
    Rivet.portal.NotificationRegisterFirebaseService
> = core.serialization.object({
    accessKey: core.serialization.property("access_key", core.serialization.string()),
});

export declare namespace NotificationRegisterFirebaseService {
    export interface Raw {
        access_key: string;
    }
}
