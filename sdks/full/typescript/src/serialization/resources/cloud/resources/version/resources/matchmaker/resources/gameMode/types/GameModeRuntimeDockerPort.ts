/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../../../index";
import * as Rivet from "../../../../../../../../../../api/index";
import * as core from "../../../../../../../../../../core";
import { PortRange as cloud_version_matchmaker_common$$portRange } from "../../common/types/PortRange";
import { PortProtocol as cloud_version_matchmaker_common$$portProtocol } from "../../common/types/PortProtocol";
import { ProxyKind as cloud_version_matchmaker_common$$proxyKind } from "../../common/types/ProxyKind";
import { cloud } from "../../../../../../../../index";

export const GameModeRuntimeDockerPort: core.serialization.ObjectSchema<
    serializers.cloud.version.matchmaker.GameModeRuntimeDockerPort.Raw,
    Rivet.cloud.version.matchmaker.GameModeRuntimeDockerPort
> = core.serialization.object({
    port: core.serialization.number().optional(),
    portRange: core.serialization.property("port_range", cloud_version_matchmaker_common$$portRange.optional()),
    protocol: cloud_version_matchmaker_common$$portProtocol.optional(),
    proxy: cloud_version_matchmaker_common$$proxyKind.optional(),
    devPort: core.serialization.property("dev_port", core.serialization.number().optional()),
    devPortRange: core.serialization.property("dev_port_range", cloud_version_matchmaker_common$$portRange.optional()),
    devProtocol: core.serialization.property("dev_protocol", cloud_version_matchmaker_common$$portProtocol.optional()),
});

export declare namespace GameModeRuntimeDockerPort {
    interface Raw {
        port?: number | null;
        port_range?: cloud.version.matchmaker.PortRange.Raw | null;
        protocol?: cloud.version.matchmaker.PortProtocol.Raw | null;
        proxy?: cloud.version.matchmaker.ProxyKind.Raw | null;
        dev_port?: number | null;
        dev_port_range?: cloud.version.matchmaker.PortRange.Raw | null;
        dev_protocol?: cloud.version.matchmaker.PortProtocol.Raw | null;
    }
}
