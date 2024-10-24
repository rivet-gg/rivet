/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { PortProtocol as actor_common$$portProtocol } from "./PortProtocol";
import { PortRouting as actor_common$$portRouting } from "./PortRouting";
import { actor } from "../../../../index";

export const Port: core.serialization.ObjectSchema<serializers.actor.Port.Raw, Rivet.actor.Port> =
    core.serialization.object({
        protocol: actor_common$$portProtocol,
        internalPort: core.serialization.property("internal_port", core.serialization.number().optional()),
        publicHostname: core.serialization.property("public_hostname", core.serialization.string().optional()),
        publicPort: core.serialization.property("public_port", core.serialization.number().optional()),
        routing: actor_common$$portRouting,
    });

export declare namespace Port {
    interface Raw {
        protocol: actor.PortProtocol.Raw;
        internal_port?: number | null;
        public_hostname?: string | null;
        public_port?: number | null;
        routing: actor.PortRouting.Raw;
    }
}
