// Generated runner protocol SDK
// Re-export namespaced types  
import { rivet as kvTypes } from './kv.js';
import { rivet as rpTypes } from './runner_protocol.js';

export const types = {
	kv: kvTypes.pegboard.kv,
	...rpTypes.pegboard.runner_protocol,
};

// Utility functions for encoding/decoding frames (same interface as JSON version)
export function encodeFrame(payload: any): Buffer {
	const protobufPayload = payload.serializeBinary();
	const payloadLength = Buffer.alloc(4);
	payloadLength.writeUInt32BE(protobufPayload.length, 0);

	const header = Buffer.alloc(4); // All zeros for now

	return Buffer.concat([payloadLength, header, Buffer.from(protobufPayload)]);
}

export function decodeFrames<T>(buffer: Buffer, MessageClass: T): T[] {
	const packets = [];
	let offset = 0;

	while (offset < buffer.length) {
		if (buffer.length - offset < 8) break; // Incomplete frame length + header
		const payloadLength = buffer.readUInt32BE(offset);
		offset += 4;

		// Skip the header (4 bytes)
		offset += 4;

		if (buffer.length - offset < payloadLength) break; // Incomplete frame data
		const payloadBuffer = buffer.subarray(offset, offset + payloadLength);
		const packet = MessageClass.deserializeBinary(new Uint8Array(payloadBuffer));
		packets.push(packet);
		offset += payloadLength;
	}

	return packets;
}
