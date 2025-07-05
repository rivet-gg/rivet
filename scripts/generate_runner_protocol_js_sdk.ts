#!/usr/bin/env -S deno run -A

import { resolve } from "@std/path";
import { existsSync } from "@std/fs";

const ROOT_DIR = resolve(import.meta.dirname!, "..");
const PROTO_DIR = resolve(ROOT_DIR, "packages/edge/infra/client/runner-protocol/resources/proto");
const OUTPUT_DIR = resolve(ROOT_DIR, "sdks/runner-protocol");

async function main() {
	console.log("Generating JS SDK for runner protocol...");

	// Check if protobuf compiler is available
	const protoc = await checkProtocAvailable();
	if (!protoc) {
		console.error("protobuf compiler (protoc) not found. Please install it first.");
		Deno.exit(1);
	}

	// Ensure output directory exists
	try {
		await Deno.mkdir(OUTPUT_DIR, { recursive: true });
	} catch (error) {
		if (!(error instanceof Deno.errors.AlreadyExists)) {
			throw error;
		}
	}

	// Generate the JS/TypeScript files
	await generateJSSDK();

	// Generate package.json
	await generatePackageJson();

	// Generate index.ts
	await generateIndexFile();

	console.log(`✅ JS SDK generated successfully in ${OUTPUT_DIR}`);
}

async function checkProtocAvailable(): Promise<boolean> {
	try {
		const process = await new Deno.Command("protoc", {
			args: ["--version"],
			stdout: "piped",
			stderr: "piped"
		}).output();
		return process.success;
	} catch {
		return false;
	}
}

async function generateJSSDK() {
	console.log("Generating protobuf JS/TS files...");

	const protoFiles = [
		"kv.proto",
		"runner_protocol.proto"
	];

	// Use the installed protoc plugins
	const nodeModulesPath = resolve(ROOT_DIR, "node_modules/.bin");
	const protocArgs = [
		"--proto_path=" + PROTO_DIR,
		"--plugin=protoc-gen-js=" + resolve(nodeModulesPath, "protoc-gen-js"),
		"--plugin=protoc-gen-ts=" + resolve(nodeModulesPath, "protoc-gen-ts"),
		"--js_out=import_style=commonjs:" + OUTPUT_DIR,
		"--ts_out=" + OUTPUT_DIR,
		...protoFiles.map(f => resolve(PROTO_DIR, f))
	];

	const process = await new Deno.Command("protoc", {
		args: protocArgs,
		cwd: ROOT_DIR,
		stdout: "piped",
		stderr: "piped"
	}).output();

	if (!process.success) {
		const stderr = new TextDecoder().decode(process.stderr);
		console.error("protoc failed:", stderr);
		Deno.exit(1);
	}

	console.log("✅ Protobuf files generated successfully");
}

async function generatePackageJson() {
	const packageJson = {
		name: "@rivet-gg/runner-protocol",
		version: "1.0.0",
		description: "JavaScript SDK for Rivet Runner Protocol",
		type: "module",
		main: "index.js",
		types: "index.d.ts",
		scripts: {
			build: "tsc",
			prepublishOnly: "npm run build"
		},
		dependencies: {
			"google-protobuf": "^3.21.0"
		},
		devDependencies: {
			"@types/google-protobuf": "^3.15.0",
			"typescript": "^5.0.0"
		},
		files: [
			"*.js",
			"*.d.ts",
			"*.ts",
			"!*.test.*"
		]
	};

	await Deno.writeTextFile(
		resolve(OUTPUT_DIR, "package.json"),
		JSON.stringify(packageJson, null, 2)
	);

	console.log("✅ package.json generated");
}

async function generateIndexFile() {
	const indexContent = `// Generated runner protocol SDK
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
`;

	await Deno.writeTextFile(
		resolve(OUTPUT_DIR, "index.ts"),
		indexContent
	);

	console.log("✅ index.ts generated");
}

if (import.meta.main) {
	await main();
}