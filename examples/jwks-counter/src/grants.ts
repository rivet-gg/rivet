import {
	AuthGrantOperationKind,
	AuthGrantResourceKind,
	type AuthGrantsData,
	encodeAuthGrants,
} from "@rivetkit/engine-data";

// Rivet IDs are 19-byte, little-endian base36 values; the first byte is the ID version.
function idBytes(id: string): ArrayBuffer {
	if (!/^[0-9a-z]{30}$/i.test(id)) throw new Error("Invalid Rivet ID");
	let value = 0n;
	for (const digit of id.toLowerCase().split("").reverse()) {
		value = value * 36n + BigInt(Number.parseInt(digit, 36));
	}
	const bytes = new Uint8Array(19);
	for (let index = 0; index < bytes.length; index++) {
		bytes[index] = Number(value & 0xffn);
		value >>= 8n;
	}
	if (value !== 0n || bytes[0] !== 1) throw new Error("Unsupported Rivet ID");
	return bytes.buffer;
}

export function encodeActorGatewayReadGrant(
	target: "any" | { id: string },
): string {
	const data: AuthGrantsData = {
		grants: [
			{
				resource: AuthGrantResourceKind.ActorGateway,
				target:
					target === "any"
						? { tag: "Any", val: null }
						: { tag: "Id", val: idBytes(target.id) },
				operations: [AuthGrantOperationKind.Read],
			},
		],
	};
	return Buffer.from(encodeAuthGrants(data)).toString("base64url");
}
