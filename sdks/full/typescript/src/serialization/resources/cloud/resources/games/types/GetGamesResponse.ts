/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Summary as game_common$$summary } from "../../../../game/resources/common/types/Summary";
import { Summary as group_common$$summary } from "../../../../group/resources/common/types/Summary";
import { WatchResponse as common$$watchResponse } from "../../../../common/types/WatchResponse";
import { game, group, common } from "../../../../index";

export const GetGamesResponse: core.serialization.ObjectSchema<
    serializers.cloud.games.GetGamesResponse.Raw,
    Rivet.cloud.games.GetGamesResponse
> = core.serialization.object({
    games: core.serialization.list(game_common$$summary),
    groups: core.serialization.list(group_common$$summary),
    watch: common$$watchResponse,
});

export declare namespace GetGamesResponse {
    interface Raw {
        games: game.Summary.Raw[];
        groups: group.Summary.Raw[];
        watch: common.WatchResponse.Raw;
    }
}
