import { createVersionedDataHandler } from "vbare";
import {
	type Data,
	decodeData,
	encodeData,
	type Grant,
	OperationKind,
	ResourceKind,
	type TargetScope,
} from "./auth-grants-v1.generated";

export {
	type Data as AuthGrantsData,
	OperationKind as AuthGrantOperationKind,
	ResourceKind as AuthGrantResourceKind,
} from "./auth-grants-v1.generated";

export const AUTH_GRANTS_VERSION = 1;

const MAX_GRANTS = 32;
const MAX_OPERATIONS_PER_GRANT = 5;
const MAX_GRANTS_BYTES = 2_048;

// This order matches the Rust authorization enums and is part of the signed v1 wire format.
const RESOURCE_ORDER = [
	ResourceKind.Namespace,
	ResourceKind.Actor,
	ResourceKind.Runner,
	ResourceKind.RunnerConfig,
	ResourceKind.Token,
	ResourceKind.Reserved,
	ResourceKind.Datacenter,
	ResourceKind.ActorGateway,
	ResourceKind.ActorKv,
] as const;
const OPERATION_ORDER = [
	OperationKind.Read,
	OperationKind.Update,
	OperationKind.List,
	OperationKind.Create,
	OperationKind.Delete,
] as const;

function rank<T>(order: readonly T[], value: T): number {
	const index = order.indexOf(value);
	if (index < 0) throw new Error("Invalid auth grant variant");
	return index;
}

function normalizeTarget(target: TargetScope): TargetScope {
	if (target.tag === "Any") return { tag: "Any", val: null };
	if (target.tag !== "Id" || !(target.val instanceof ArrayBuffer))
		throw new Error("Invalid auth grant target");
	const bytes = new Uint8Array(target.val);
	if (bytes.length !== 19 || bytes[0] !== 1)
		throw new Error("Invalid auth grant target ID");
	return { tag: "Id", val: bytes.slice().buffer };
}

function compareSelector(a: Grant, b: Grant): number {
	const resource =
		rank(RESOURCE_ORDER, a.resource) - rank(RESOURCE_ORDER, b.resource);
	if (resource !== 0) return resource;
	if (a.target.tag !== b.target.tag) return a.target.tag === "Any" ? -1 : 1;
	if (a.target.tag === "Any" || b.target.tag === "Any") return 0;
	const left = new Uint8Array(a.target.val);
	const right = new Uint8Array(b.target.val);
	for (let index = 1; index < left.length; index++) {
		if (left[index] !== right[index]) return left[index] - right[index];
	}
	return 0;
}

function canonicalize(data: Data): Data {
	if (
		!Array.isArray(data.grants) ||
		data.grants.length === 0 ||
		data.grants.length > MAX_GRANTS
	)
		throw new Error("Invalid auth grant count");
	const sorted = data.grants.map((grant): Grant => {
		if (
			grant.resource === ResourceKind.Token ||
			grant.resource === ResourceKind.Reserved
		)
			throw new Error("Control-plane auth grants are not allowed");
		rank(RESOURCE_ORDER, grant.resource);
		if (
			!Array.isArray(grant.operations) ||
			grant.operations.length === 0 ||
			grant.operations.length > MAX_OPERATIONS_PER_GRANT
		)
			throw new Error("Invalid auth grant operations");
		for (const operation of grant.operations)
			rank(OPERATION_ORDER, operation);
		return {
			resource: grant.resource,
			target: normalizeTarget(grant.target),
			operations: [...new Set<OperationKind>(grant.operations)].sort(
				(a, b) => rank(OPERATION_ORDER, a) - rank(OPERATION_ORDER, b),
			),
		};
	});
	sorted.sort(compareSelector);
	const grants: Grant[] = [];
	for (const grant of sorted) {
		const previous = grants.at(-1);
		if (previous && compareSelector(previous, grant) === 0) {
			const operations = [
				...new Set([...previous.operations, ...grant.operations]),
			].sort(
				(a, b) => rank(OPERATION_ORDER, a) - rank(OPERATION_ORDER, b),
			);
			if (operations.length > MAX_OPERATIONS_PER_GRANT)
				throw new Error("Too many auth grant operations");
			grants[grants.length - 1] = { ...previous, operations };
		} else {
			grants.push(grant);
		}
	}
	return { grants };
}

const grantsData = createVersionedDataHandler<Data>({
	deserializeVersion: (bytes, version) => {
		if (version !== AUTH_GRANTS_VERSION)
			throw new Error("Unsupported grants version");
		return decodeData(bytes);
	},
	serializeVersion: (data, version) => {
		if (version !== AUTH_GRANTS_VERSION)
			throw new Error("Unsupported grants version");
		return encodeData(data);
	},
	deserializeConverters: () => [],
	serializeConverters: () => [],
});

/**
 * Canonicalizes and encodes the versioned BARE payload for `rivet_grants`.
 * The token's signed `rivet_ns` claim supplies the namespace; grants do not embed it.
 */
export function encodeAuthGrants(data: Data): Uint8Array {
	const encoded = grantsData.serializeWithEmbeddedVersion(
		canonicalize(data),
		AUTH_GRANTS_VERSION,
	);
	if (encoded.length > MAX_GRANTS_BYTES)
		throw new Error("Auth grants exceed the maximum encoded size");
	return encoded;
}
