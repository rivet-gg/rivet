import { gunzipSync } from "node:zlib";

export interface TarEntry {
	readonly path: string;
	readonly size: number;
	readonly data: Uint8Array;
}

/** Reads the regular files of a gzipped tarball, such as an npm package. */
export function readTarball(gzipped: Uint8Array): Array<TarEntry> {
	const tar = gunzipSync(gzipped);
	const entries: Array<TarEntry> = [];
	let offset = 0;
	while (offset + 512 <= tar.length) {
		const header = tar.subarray(offset, offset + 512);
		if (header.every((byte) => byte === 0)) break;
		const name = field(header, 0, 100);
		const prefix = field(header, 345, 155);
		const size = Number.parseInt(field(header, 124, 12).trim() || "0", 8);
		const type = String.fromCharCode(header[156] ?? 0);
		const start = offset + 512;
		if (type === "0" || type === "\0") {
			entries.push({ path: prefix ? `${prefix}/${name}` : name, size, data: tar.subarray(start, start + size) });
		}
		offset = start + Math.ceil(size / 512) * 512;
	}
	return entries;
}

/** Unpacked size: the sum of file sizes, the same thing npm reports as `unpackedSize`. */
export function unpackedSize(gzipped: Uint8Array): number {
	return readTarball(gzipped).reduce((sum, entry) => sum + entry.size, 0);
}

function field(header: Uint8Array, start: number, length: number): string {
	const bytes = header.subarray(start, start + length);
	const end = bytes.indexOf(0);
	return new TextDecoder().decode(end === -1 ? bytes : bytes.subarray(0, end));
}
