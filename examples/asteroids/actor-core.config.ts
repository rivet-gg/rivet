import type { Config } from "@actor-core/rivet";
import { lobbyManager } from "@actor-core/lobby-manager";
import { setup } from "actor-core";
import { RIVET_SERVICE_TOKEN } from "./token.secret.ts";
import { app } from "./actor-core";

export default { app } satisfies Config;

//engine:
//  html5: {}
//matchmaker:
//  max_players: 32
//  tier: "basic-1d2"
//  regions:
//    atl: {}
//    fra: {}
//  game_modes:
//    default: {}
//    bullet-hell: {}
//  docker:
//    dockerfile: "Dockerfile"
//    ports:
//      default:
//        port: 3000
//        protocol: "https"
//cdn:
//  build_command: "yarn install && yarn run build:client"
//  build_output: "build/client/"
//scripts:
//  server: "npx nodemon -r dotenv/config --delay 1 --watch server --watch shared --ext ts --exec ts-node server/index.ts"
//  server:inspect: "npx nodemon -r dotenv/config -r ts-node/register --exec node --inspect-brk -r ts-node/register server/index.ts"
//  client: "npx vite --host 0.0.0.0 --port 8080"
