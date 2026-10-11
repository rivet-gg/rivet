import assert from "node:assert/strict";
import {
	AuthGrantOperationKind,
	AuthGrantResourceKind,
	encodeAuthGrants,
} from "../src/index";

assert.deepEqual(
	[
		...encodeAuthGrants({
			grants: [
				{
					resource: AuthGrantResourceKind.ActorGateway,
					target: { tag: "Any", val: null },
					operations: [AuthGrantOperationKind.Read],
				},
			],
		}),
	],
	[1, 0, 1, 7, 0, 1, 0],
);

const idA = Uint8Array.from([1, 1, ...Array(17).fill(0)]).buffer;
const idB = Uint8Array.from([1, 2, ...Array(17).fill(0)]).buffer;
const canonical = encodeAuthGrants({
	grants: [
		{
			resource: AuthGrantResourceKind.Runner,
			target: { tag: "Any", val: null },
			operations: [AuthGrantOperationKind.Delete],
		},
		{
			resource: AuthGrantResourceKind.Actor,
			target: { tag: "Id", val: idB },
			operations: [AuthGrantOperationKind.Create],
		},
		{
			resource: AuthGrantResourceKind.Actor,
			target: { tag: "Id", val: idA },
			operations: [
				AuthGrantOperationKind.List,
				AuthGrantOperationKind.Read,
				AuthGrantOperationKind.Read,
			],
		},
		{
			resource: AuthGrantResourceKind.Actor,
			target: { tag: "Any", val: null },
			operations: [
				AuthGrantOperationKind.Update,
				AuthGrantOperationKind.Read,
			],
		},
		{
			resource: AuthGrantResourceKind.Actor,
			target: { tag: "Id", val: idA },
			operations: [AuthGrantOperationKind.Update],
		},
	],
});
assert.equal(
	Buffer.from(canonical).toString("base64url"),
	"AQAEAQACAAEBARMBAQAAAAAAAAAAAAAAAAAAAAAAAwABAgEBEwECAAAAAAAAAAAAAAAAAAAAAAABAwIAAQQ",
);

assert.throws(() => encodeAuthGrants({ grants: [] }));
assert.throws(() =>
	encodeAuthGrants({
		grants: [
			{
				resource: AuthGrantResourceKind.Token,
				target: { tag: "Any", val: null },
				operations: [AuthGrantOperationKind.Create],
			},
		],
	}),
);
assert.throws(() =>
	encodeAuthGrants({
		grants: [
			{
				resource: AuthGrantResourceKind.Actor,
				target: { tag: "Id", val: new ArrayBuffer(18) },
				operations: [AuthGrantOperationKind.Read],
			},
		],
	}),
);
assert.throws(() =>
	encodeAuthGrants({
		grants: [
			{
				resource: AuthGrantResourceKind.Actor,
				target: { tag: "Any", val: null },
				operations: Array(6).fill(AuthGrantOperationKind.Read),
			},
		],
	}),
);

console.log("Versioned auth grants encoding passed");

assert.throws(() =>
	encodeAuthGrants({
		grants: [
			{
				resource: AuthGrantResourceKind.Reserved,
				target: { tag: "Any", val: null },
				operations: [AuthGrantOperationKind.Read],
			},
		],
	}),
);
