# BetterStack Monitoring

Ships container metrics & logs to BetterStack. Uses Vector to collect & ship logs under the hood.

## How it works

Vector will run as the parent process to your main process. It will colelct logs & send all metrics to BetterStack automatically.

The BetterStack information will be configured for the Actor using the environment variables `BETTERSTACK_HOST` and `BETTERSTACK_TOKEN`.

## Prerequisites

- [Rivet CLI](https://rivet.gg/docs/setup)
- [Docker](https://docs.docker.com/desktop/)
- [Node.js](https://nodejs.org/) (for local development)

## File structure

- `rivet.json` - Configuration file for deploying the service
- `Dockerfile` - Dockerfile that will automatically be deployed
- `server.js` - Hono HTTP server implementation
- `package.json` - Node.js dependencies

## Local Development

```sh
yarn install
PORT_HTTP=8080 yarn start
```

## Testing

### Using Docker

```sh
docker build -t bs-actor . && docker run -e BETTERSTACK_TOKEN=<TOKEN> -e BETTERSTACK_HOST=<HOST> bs-actor
```

### Using e2e_test.js

```sh
RIVET_SERVICE_TOKEN=<TOKEN> RIVET_PROJECT=<PROJECT> RIVET_ENVIRONMENT=<ENV> BETTERSTACK_TOKEN=<TOKEN> BETTERSTACK_HOST=<HOST> yarn test
```

## Deploying

```sh
rivet deploy
```

