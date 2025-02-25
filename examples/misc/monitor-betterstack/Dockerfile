# syntax=docker/dockerfile:1
# check=skip=SecretsUsedInArgOrEnv

FROM node:22-bookworm-slim

# Set environment variables
ENV CI=true
ENV COREPACK_ENABLE_STRICT=0
ENV VECTOR_LOG=WARN
RUN corepack enable

# Install Vector
RUN apt-get update -y && \
	apt-get install -y curl && \
	bash -c "$(curl -L https://setup.vector.dev)" && \
	apt-get install -y vector

# Copy configuration files
COPY ./.env* /app/.env*
COPY ./vector.yaml /etc/vector/vector.yaml

# Create a non-root user
RUN groupadd -r rivet && useradd -r -g rivet rivet

WORKDIR /app
COPY . .
RUN yarn install && \
    chown -R rivet:rivet /app

# Switch to non-root user
USER rivet

# Vector will spawn your server as a child process
CMD ["sh", "-c", "NODE_ENV=production node -r dotenv/config ./server.js 2>&1 | vector -vv"]

