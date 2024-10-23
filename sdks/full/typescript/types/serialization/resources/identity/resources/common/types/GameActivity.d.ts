/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { game } from "../../../../index";
export declare const GameActivity: core.serialization.ObjectSchema<serializers.identity.GameActivity.Raw, Rivet.identity.GameActivity>;
export declare namespace GameActivity {
    interface Raw {
        game: game.Handle.Raw;
        message: string;
        public_metadata?: unknown | null;
        mutual_metadata?: unknown | null;
    }
}