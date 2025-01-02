# Rivet Actors

Powered by [Rivet](https://rivet.gg).

## Overview

### Simple Chat

It's a simple chat implementation using Rivet's `Actor` API. It demonstrates how to use the `Actor` API to
create a chat application. The chat application is a simple chat room where users can send messages and see
messages from other users in real-time. **Does not include any authentication or authorization.**

### Server Driven UI

It's a simple React Server Component implementation using Rivet's `Actor` API. It demonstrates how to use the
`Actor` API to create a server-driven UI. The server component renders a simple response with passed props,
and updates the UI in real-time.

This example uses RSC payload introduced in the newest version of React. It's not widely supported yet, and
it's only available in a few frameworks: Next.js or Waku. That's why we need those additionals steps to make
it work in Deno. We hope that in the future this example will be easier to implement / cleaner.

### Preqrequisites

- Compile React dependencies to support Server Components in Deno (only needed once)
  ```sh
  cd scripts; deno run -A compile_react_deps.ts
  ```

We need to compile `react-server-dom-webpack/server`, by hand as Deno does not support import conditions yet.

## Prerequisites

- [Rivet CLI](https://rivet.gg/docs/setup)
- [Deno](https://deno.land/)

## File structure

- `rivet.json` Configuration file for deploying the actor
- `deno.json` Configuration file for dependencies for the actor
- `simple-chat.ts` Simple chat implementation
- `server-driven-ui.ts` Server Driven UI using RSC payload

## Deploying

```sh
rivet login
rivet deploy
```
