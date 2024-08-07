/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../../../index";
import * as Rivet from "../../../../../../../../../../api/index";
import * as core from "../../../../../../../../../../core";
import { LogsLobbySummary as cloud_common$$logsLobbySummary } from "../../../../../../common/types/LogsLobbySummary";
import { SvcMetrics as cloud_common$$svcMetrics } from "../../../../../../common/types/SvcMetrics";
import { SvcPerf as cloud_common$$svcPerf } from "../../../../../../common/types/SvcPerf";
import { cloud } from "../../../../../../../../index";

export const GetNamespaceLobbyResponse: core.serialization.ObjectSchema<
    serializers.cloud.games.namespaces.GetNamespaceLobbyResponse.Raw,
    Rivet.cloud.games.namespaces.GetNamespaceLobbyResponse
> = core.serialization.object({
    lobby: cloud_common$$logsLobbySummary,
    metrics: cloud_common$$svcMetrics.optional(),
    stdoutPresignedUrls: core.serialization.property(
        "stdout_presigned_urls",
        core.serialization.list(core.serialization.string())
    ),
    stderrPresignedUrls: core.serialization.property(
        "stderr_presigned_urls",
        core.serialization.list(core.serialization.string())
    ),
    perfLists: core.serialization.property("perf_lists", core.serialization.list(cloud_common$$svcPerf)),
});

export declare namespace GetNamespaceLobbyResponse {
    interface Raw {
        lobby: cloud.LogsLobbySummary.Raw;
        metrics?: cloud.SvcMetrics.Raw | null;
        stdout_presigned_urls: string[];
        stderr_presigned_urls: string[];
        perf_lists: cloud.SvcPerf.Raw[];
    }
}
