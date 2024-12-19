// DO NOT MODIFY
//
// Generated from sdks/actors-bridge/

import type { Metadata } from "./types/metadata.js";
export type { Metadata } from "./types/metadata.js";
export declare function deepFreeze<T extends object>(object: T): Readonly<T>;
export declare const ACTOR_CONTEXT: {
	metadata: Metadata;
	kv: {
		get: <K, V>(
			key: K,
			options?: import("./40_rivet_kv.js").GetOptions,
		) => Promise<V | null>;
		getBatch: <K extends Array<unknown>, V>(
			keys: K,
			options?: import("./40_rivet_kv.js").GetBatchOptions,
		) => Promise<Map<K[number], V>>;
		list: <K, V>(
			options?: import("./40_rivet_kv.js").ListOptions<K>,
		) => Promise<Map<K, V>>;
		put: <K, V>(
			key: K,
			value: V | ArrayBuffer,
			options?: import("./40_rivet_kv.js").PutOptions,
		) => Promise<void>;
		putBatch: <K, V>(
			obj: Map<K, V | ArrayBuffer>,
			options?: import("./40_rivet_kv.js").PutBatchOptions,
		) => Promise<void>;
		delete: <K>(key: K) => Promise<void>;
		deleteBatch: <K extends Array<unknown>>(keys: K) => Promise<void>;
		deleteAll: () => Promise<void>;
	};
};
