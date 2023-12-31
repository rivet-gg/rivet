/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";

export const Handle: core.serialization.ObjectSchema<serializers.group.Handle.Raw, Rivet.group.Handle> =
    core.serialization.object({
        groupId: core.serialization.property("group_id", core.serialization.string()),
        displayName: core.serialization.property(
            "display_name",
            core.serialization.lazy(async () => (await import("../../../../..")).DisplayName)
        ),
        avatarUrl: core.serialization.property("avatar_url", core.serialization.string().optional()),
        external: core.serialization.lazyObject(async () => (await import("../../../../..")).group.ExternalLinks),
        isDeveloper: core.serialization.property("is_developer", core.serialization.boolean().optional()),
    });

export declare namespace Handle {
    interface Raw {
        group_id: string;
        display_name: serializers.DisplayName.Raw;
        avatar_url?: string | null;
        external: serializers.group.ExternalLinks.Raw;
        is_developer?: boolean | null;
    }
}
