/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as Rivet from "../../../../../../index";
/**
 * @example
 *     {
 *         lobbyId: "string",
 *         captcha: {
 *             hcaptcha: {},
 *             turnstile: {}
 *         },
 *         verificationData: {
 *             "key": "value"
 *         }
 *     }
 */
export interface JoinLobbyRequest {
    lobbyId: string;
    captcha?: Rivet.captcha.Config;
    verificationData?: unknown;
}
