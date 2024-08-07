/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common, group } from "../../../../index";
export declare const Profile: core.serialization.ObjectSchema<serializers.group.Profile.Raw, Rivet.group.Profile>;
export declare namespace Profile {
    interface Raw {
        group_id: string;
        display_name: common.DisplayName.Raw;
        avatar_url?: string | null;
        external: group.ExternalLinks.Raw;
        is_developer?: boolean | null;
        bio: string;
        is_current_identity_member?: boolean | null;
        publicity: group.Publicity.Raw;
        member_count?: number | null;
        members: group.Member.Raw[];
        join_requests: group.JoinRequest.Raw[];
        is_current_identity_requesting_join?: boolean | null;
        owner_identity_id: string;
    }
}
