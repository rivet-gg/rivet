/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { DisplayName } from "../../../../common/types/DisplayName";
import { ExternalLinks } from "./ExternalLinks";

export const Handle: core.serialization.ObjectSchema<serializers.group.Handle.Raw, Rivet.group.Handle> =
    core.serialization.object({
        groupId: core.serialization.property("group_id", core.serialization.string()),
        displayName: core.serialization.property("display_name", DisplayName),
        avatarUrl: core.serialization.property("avatar_url", core.serialization.string().optional()),
        external: ExternalLinks,
        isDeveloper: core.serialization.property("is_developer", core.serialization.boolean().optional()),
    });

export declare namespace Handle {
    export interface Raw {
        group_id: string;
        display_name: DisplayName.Raw;
        avatar_url?: string | null;
        external: ExternalLinks.Raw;
        is_developer?: boolean | null;
    }
}
