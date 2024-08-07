/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common, group, game } from "../../../../index";
export declare const Profile: core.serialization.ObjectSchema<serializers.game.Profile.Raw, Rivet.game.Profile>;
export declare namespace Profile {
    interface Raw {
        game_id: string;
        name_id: string;
        display_name: common.DisplayName.Raw;
        logo_url?: string | null;
        banner_url?: string | null;
        url: string;
        developer: group.Summary.Raw;
        tags: string[];
        description: string;
        platforms: game.PlatformLink.Raw[];
        recommended_groups: group.Summary.Raw[];
        identity_leaderboard_categories: game.LeaderboardCategory.Raw[];
        group_leaderboard_categories: game.LeaderboardCategory.Raw[];
    }
}
