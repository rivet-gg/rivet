/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const JoinPlayer: core.serialization.ObjectSchema<serializers.matchmaker.JoinPlayer.Raw, Rivet.matchmaker.JoinPlayer>;
export declare namespace JoinPlayer {
    interface Raw {
        token: serializers.Jwt.Raw;
    }
}