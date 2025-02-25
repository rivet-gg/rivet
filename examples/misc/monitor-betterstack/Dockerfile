# syntax=docker/dockerfile:1
# check=skip=SecretsUsedInArgOrEnv

FROM node:22-bookworm-slim

# BetterStack credentials as build arguments - required
ARG BETTERSTACK_TOKEN
ARG BETTERSTACK_HOST
RUN test -n "$BETTERSTACK_TOKEN" || { echo "BETTERSTACK_TOKEN build argument is required"; exit 1; } && \
    test -n "$BETTERSTACK_HOST" || { echo "BETTERSTACK_HOST build argument is required"; exit 1; }

# Set environment variables including BetterStack credentials
ENV CI=true
ENV COREPACK_ENABLE_STRICT=0
ENV VECTOR_LOG=WARN
ENV BETTERSTACK_TOKEN=${BETTERSTACK_TOKEN}
ENV BETTERSTACK_HOST=${BETTERSTACK_HOST}
RUN corepack enable

# Install Vector
RUN apt-get update -y && \
	apt-get install -y curl && \
	bash -c "$(curl -L https://setup.vector.dev)" && \
	apt-get install -y vector

# Configure Vector with BetterStack
COPY ./vector.yaml /etc/vector/vector.yaml

# Create a non-root user
RUN groupadd -r rivet && useradd -r -g rivet rivet
USER rivet

WORKDIR /app
COPY --chown=rivet:rivet . .
RUN yarn install

# Vector will spawn your server as a child process
CMD ["sh", "-c", "NODE_ENV=production node ./server.js 2>&1 | vector -vv"]

