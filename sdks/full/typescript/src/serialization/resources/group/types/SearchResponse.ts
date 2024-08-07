/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { Handle as group_common$$handle } from "../resources/common/types/Handle";
import { group } from "../../index";

export const SearchResponse: core.serialization.ObjectSchema<
    serializers.group.SearchResponse.Raw,
    Rivet.group.SearchResponse
> = core.serialization.object({
    groups: core.serialization.list(group_common$$handle),
    anchor: core.serialization.string().optional(),
});

export declare namespace SearchResponse {
    interface Raw {
        groups: group.Handle.Raw[];
        anchor?: string | null;
    }
}
