# syntax=docker/dockerfile:1
# check=skip=SecretsUsedInArgOrEnv

FROM node:22-bookworm-slim

# BetterStack credentials as build arguments with defaults
ARG BETTERSTACK_TOKEN="your_token_here"
ARG BETTERSTACK_HOST="logs.betterstack.com"

# Set environment variables including BetterStack credentials
ENV CI=true
ENV COREPACK_ENABLE_STRICT=0
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

WORKDIR /app
COPY . .
RUN yarn install && \
    chown -R rivet:rivet /app

# Switch to non-root user
USER rivet

# Add healthcheck
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Add labels for better container management
LABEL org.opencontainers.image.description="BetterStack monitoring example"
LABEL org.opencontainers.image.source="https://github.com/rivet-gg/examples"

# Vector will spawn your server as a child process
ENV VECTOR_LOG=WARN
CMD ["sh", "-c", "NODE_ENV=production node ./server.js 2>&1 | vector -vv"]

