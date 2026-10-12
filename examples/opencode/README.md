# OpenCode on Rivet Actors

Run OpenCode v2 inside a Rivet Actor. The actor owns an embedded OpenCode host,
stores its sessions and events in actor SQLite, and restores unfinished work when
it wakes.

## Getting Started

From this directory, install dependencies and build the server:

```sh
pnpm install
pnpm build
```

In a fresh monorepo checkout, build the linked RivetKit JavaScript package and
native runtime before starting this example. The example build bundles OpenCode;
it does not build RivetKit's native addon.

Set a provider key and the directory OpenCode may work in, then start the server:

```sh
export ANTHROPIC_API_KEY=...
export OPENCODE_WORKDIR=/path/to/your/project
pnpm start
```

In another terminal, send a prompt to the `demo` actor:

```sh
pnpm client "Explain the project structure"
```

Use a different key in `src/client.ts` to create an independent OpenCode session.

## Features

- One embedded OpenCode host per Rivet Actor, created on wake and closed on sleep.
- OpenCode sessions and events stored in the actor's SQLite database.
- Unfinished sessions resumed after the actor wakes.
- Prompt, message history, wait, and abort actions exposed to a server-side client.

## Prerequisites

- An OpenCode-supported model provider key, such as `ANTHROPIC_API_KEY`.
- A local or mounted project directory for `OPENCODE_WORKDIR`.

## Implementation

[`src/actor.ts`](https://github.com/rivet-dev/rivet/tree/main/examples/opencode/src/actor.ts)
owns the actor lifecycle and actions.
[`src/host.ts`](https://github.com/rivet-dev/rivet/tree/main/examples/opencode/src/host.ts)
embeds OpenCode, and
[`src/sqlite.ts`](https://github.com/rivet-dev/rivet/tree/main/examples/opencode/src/sqlite.ts)
adapts OpenCode's SQL driver to Rivet's actor SQLite transactions.
[`src/client.ts`](https://github.com/rivet-dev/rivet/tree/main/examples/opencode/src/client.ts)
shows the prompt flow.

OpenCode tools use the server process's filesystem and OS permissions. Session
records are durable, but files under `OPENCODE_WORKDIR` need their own persistent
storage if the actor moves to another machine. This example does not provide
process isolation or filesystem sandboxing. Keep its actor actions behind your
application's normal authentication and authorization.

## Resources

Read more about [actor SQLite](https://rivet.dev/docs/actors/sqlite),
[lifecycle hooks](https://rivet.dev/docs/actors/lifecycle), and
[actions](https://rivet.dev/docs/actors/actions).

## License

MIT
