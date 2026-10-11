// Bundle with esbuild for Node, then run the result. This exercises real HTTP transport.
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { RivetClient } from "../../../engine/sdks/typescript/api-full/src/index";
for (const body of [Buffer.alloc(0), Buffer.from([0]), Buffer.from([255,254,128]), Buffer.from('\r\n{ "a": 1 }\r\n')]) {
    const server = createServer(async (request, response) => {
        const chunks = [];
        for await (const chunk of request) chunks.push(chunk);
        assert.deepEqual(Buffer.concat(chunks), body);
        assert.equal(request.url, "/auth/webhooks/sign");
        assert.equal(request.headers["content-type"], "application/octet-stream");
        assert.equal(request.headers.authorization, "Bearer sdk-test");
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify({timestamp:"123", kid:"ds_test", signature:"test"}));
    });
    await new Promise<void>(resolve => server.listen(0, "127.0.0.1", resolve));
    try {
        const address = server.address(); assert.ok(address && typeof address !== "string");
        const client = new RivetClient({environment:`http://127.0.0.1:${address.port}`, token:"sdk-test"});
        const response = await client.authWebhookSigning.authWebhooksSign(new Blob([body]), {maxRetries:0});
        assert.equal(response.timestamp, "123");
    } finally { await new Promise<void>(resolve => server.close(() => resolve())); }
}
console.log("Generated TypeScript client preserves all four byte fixtures");
