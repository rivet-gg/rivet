/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const Config: core.serialization.ObjectSchema<serializers.cloud.version.matchmaker.Config.Raw, Rivet.cloud.version.matchmaker.Config>;
export declare namespace Config {
    interface Raw {
        game_modes?: Record<string, serializers.cloud.version.matchmaker.GameMode.Raw> | null;
        captcha?: serializers.cloud.version.matchmaker.Captcha.Raw | null;
        dev_hostname?: string | null;
        regions?: Record<string, serializers.cloud.version.matchmaker.GameModeRegion.Raw> | null;
        max_players?: number | null;
        max_players_direct?: number | null;
        max_players_party?: number | null;
        docker?: serializers.cloud.version.matchmaker.GameModeRuntimeDocker.Raw | null;
        tier?: string | null;
        idle_lobbies?: serializers.cloud.version.matchmaker.GameModeIdleLobbiesConfig.Raw | null;
        lobby_groups?: serializers.cloud.version.matchmaker.LobbyGroup.Raw[] | null;
    }
}