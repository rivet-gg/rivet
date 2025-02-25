# BetterStack Monitoring

Ships container metrics & logs to BetterStack. Uses Vector to collect & ship logs under the hood.

## How it works

Vector will run as the parent process to your main process. It will collect logs & send all metrics to BetterStack automatically.

The BetterStack credentials are hardcoded in the Dockerfile as build arguments and set as environment variables. You can override them during build time.

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
# Build with default credentials
docker build -t bs-actor .
docker run bs-actor

# Or with custom BetterStack credentials
docker build -t bs-actor --build-arg BETTERSTACK_TOKEN=<TOKEN> .
```

### Using e2e_test.js

```sh
RIVET_SERVICE_TOKEN=<TOKEN> RIVET_PROJECT=<PROJECT> RIVET_ENVIRONMENT=<ENV> yarn test
```

## Deploying

```sh
rivet deploy
```

