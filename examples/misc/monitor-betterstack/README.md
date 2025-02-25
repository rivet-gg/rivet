# BetterStack Monitoring

Ships container metrics & logs to BetterStack. Uses Vector to collect & ship logs under the hood.

## How it works

Vector will run as the parent process to your main process. It will collect logs & send all metrics to BetterStack automatically.

The BetterStack credentials are hardcoded in the Dockerfile as build arguments and set as environment variables. You can override them during build time.

## Prerequisites

- [Rivet CLI](https://rivet.gg/docs/setup)
- [Docker](https://docs.docker.com/desktop/)
- [Node.js](https://nodejs.org/) (for local development)

## Local Development

```sh
yarn install
PORT_HTTP=8080 yarn start
```

## Testing

### Using Docker

```sh
docker build --build-arg BETTERSTACK_HOST=<HOST> --build-arg BETTERSTACK_TOKEN=<TOKEN> -t bs-actor .
docker run bs-actor
```

### Using e2e_test.js

```sh
rivet deploy
RIVET_SERVICE_TOKEN=<TOKEN> RIVET_PROJECT=<PROJECT> RIVET_ENVIRONMENT=<ENV> yarn test
```

