import * as bare from "@bare-ts/lib"

const DEFAULT_CONFIG = /* @__PURE__ */ bare.Config({})

export type Any = null

export type Id = ArrayBuffer

export function readId(bc: bare.ByteCursor): Id {
    return bare.readData(bc)
}

export function writeId(bc: bare.ByteCursor, x: Id): void {
    bare.writeData(bc, x)
}

export type TargetScope =
    | { readonly tag: "Any"; readonly val: Any }
    | { readonly tag: "Id"; readonly val: Id }

export function readTargetScope(bc: bare.ByteCursor): TargetScope {
    const offset = bc.offset
    const tag = bare.readU8(bc)
    switch (tag) {
        case 0:
            return { tag: "Any", val: null }
        case 1:
            return { tag: "Id", val: readId(bc) }
        default: {
            bc.offset = offset
            throw new bare.BareError(offset, "invalid tag")
        }
    }
}

export function writeTargetScope(bc: bare.ByteCursor, x: TargetScope): void {
    switch (x.tag) {
        case "Any": {
            bare.writeU8(bc, 0)
            break
        }
        case "Id": {
            bare.writeU8(bc, 1)
            writeId(bc, x.val)
            break
        }
    }
}

export enum ResourceKind {
    Namespace = "Namespace",
    Actor = "Actor",
    Runner = "Runner",
    RunnerConfig = "RunnerConfig",
    Token = "Token",
    Reserved = "Reserved",
    Datacenter = "Datacenter",
    ActorGateway = "ActorGateway",
    ActorKv = "ActorKv",
}

export function readResourceKind(bc: bare.ByteCursor): ResourceKind {
    const offset = bc.offset
    const tag = bare.readU8(bc)
    switch (tag) {
        case 0:
            return ResourceKind.Namespace
        case 1:
            return ResourceKind.Actor
        case 2:
            return ResourceKind.Runner
        case 3:
            return ResourceKind.RunnerConfig
        case 4:
            return ResourceKind.Token
        case 5:
            return ResourceKind.Reserved
        case 6:
            return ResourceKind.Datacenter
        case 7:
            return ResourceKind.ActorGateway
        case 8:
            return ResourceKind.ActorKv
        default: {
            bc.offset = offset
            throw new bare.BareError(offset, "invalid tag")
        }
    }
}

export function writeResourceKind(bc: bare.ByteCursor, x: ResourceKind): void {
    switch (x) {
        case ResourceKind.Namespace: {
            bare.writeU8(bc, 0)
            break
        }
        case ResourceKind.Actor: {
            bare.writeU8(bc, 1)
            break
        }
        case ResourceKind.Runner: {
            bare.writeU8(bc, 2)
            break
        }
        case ResourceKind.RunnerConfig: {
            bare.writeU8(bc, 3)
            break
        }
        case ResourceKind.Token: {
            bare.writeU8(bc, 4)
            break
        }
        case ResourceKind.Reserved: {
            bare.writeU8(bc, 5)
            break
        }
        case ResourceKind.Datacenter: {
            bare.writeU8(bc, 6)
            break
        }
        case ResourceKind.ActorGateway: {
            bare.writeU8(bc, 7)
            break
        }
        case ResourceKind.ActorKv: {
            bare.writeU8(bc, 8)
            break
        }
    }
}

export enum OperationKind {
    Read = "Read",
    Update = "Update",
    List = "List",
    Create = "Create",
    Delete = "Delete",
}

export function readOperationKind(bc: bare.ByteCursor): OperationKind {
    const offset = bc.offset
    const tag = bare.readU8(bc)
    switch (tag) {
        case 0:
            return OperationKind.Read
        case 1:
            return OperationKind.Update
        case 2:
            return OperationKind.List
        case 3:
            return OperationKind.Create
        case 4:
            return OperationKind.Delete
        default: {
            bc.offset = offset
            throw new bare.BareError(offset, "invalid tag")
        }
    }
}

export function writeOperationKind(bc: bare.ByteCursor, x: OperationKind): void {
    switch (x) {
        case OperationKind.Read: {
            bare.writeU8(bc, 0)
            break
        }
        case OperationKind.Update: {
            bare.writeU8(bc, 1)
            break
        }
        case OperationKind.List: {
            bare.writeU8(bc, 2)
            break
        }
        case OperationKind.Create: {
            bare.writeU8(bc, 3)
            break
        }
        case OperationKind.Delete: {
            bare.writeU8(bc, 4)
            break
        }
    }
}

function read0(bc: bare.ByteCursor): readonly OperationKind[] {
    const len = bare.readUintSafe(bc)
    if (len === 0) {
        return []
    }
    const result = [readOperationKind(bc)]
    for (let i = 1; i < len; i++) {
        result[i] = readOperationKind(bc)
    }
    return result
}

function write0(bc: bare.ByteCursor, x: readonly OperationKind[]): void {
    bare.writeUintSafe(bc, x.length)
    for (let i = 0; i < x.length; i++) {
        writeOperationKind(bc, x[i])
    }
}

export type Grant = {
    readonly resource: ResourceKind
    readonly target: TargetScope
    readonly operations: readonly OperationKind[]
}

export function readGrant(bc: bare.ByteCursor): Grant {
    return {
        resource: readResourceKind(bc),
        target: readTargetScope(bc),
        operations: read0(bc),
    }
}

export function writeGrant(bc: bare.ByteCursor, x: Grant): void {
    writeResourceKind(bc, x.resource)
    writeTargetScope(bc, x.target)
    write0(bc, x.operations)
}

function read1(bc: bare.ByteCursor): readonly Grant[] {
    const len = bare.readUintSafe(bc)
    if (len === 0) {
        return []
    }
    const result = [readGrant(bc)]
    for (let i = 1; i < len; i++) {
        result[i] = readGrant(bc)
    }
    return result
}

function write1(bc: bare.ByteCursor, x: readonly Grant[]): void {
    bare.writeUintSafe(bc, x.length)
    for (let i = 0; i < x.length; i++) {
        writeGrant(bc, x[i])
    }
}

export type Data = {
    readonly grants: readonly Grant[]
}

export function readData(bc: bare.ByteCursor): Data {
    return {
        grants: read1(bc),
    }
}

export function writeData(bc: bare.ByteCursor, x: Data): void {
    write1(bc, x.grants)
}

export function encodeData(x: Data, config?: Partial<bare.Config>): Uint8Array {
    const fullConfig = config != null ? bare.Config(config) : DEFAULT_CONFIG
    const bc = new bare.ByteCursor(
        new Uint8Array(fullConfig.initialBufferLength),
        fullConfig,
    )
    writeData(bc, x)
    return new Uint8Array(bc.view.buffer, bc.view.byteOffset, bc.offset)
}

export function decodeData(bytes: Uint8Array): Data {
    const bc = new bare.ByteCursor(bytes, DEFAULT_CONFIG)
    const result = readData(bc)
    if (bc.offset < bc.view.byteLength) {
        throw new bare.BareError(bc.offset, "remaining bytes")
    }
    return result
}
