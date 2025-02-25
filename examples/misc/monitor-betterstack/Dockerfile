# syntax=docker/dockerfile:1.4-labs
# no-warn-secret

FROM node:22-bookworm-slim

ENV CI=true
ENV COREPACK_ENABLE_STRICT=0
RUN corepack enable

# Install Vector
RUN apt-get update -y && \
	apt-get install -y curl && \
	bash -c "$(curl -L https://setup.vector.dev)" && \
	apt-get install -y vector

# Configure Vector with BetterStack
COPY ./vector.yaml /etc/vector/vector.yaml

WORKDIR /app
COPY . .
RUN yarn install

# Vector will spawn your server as a child process
ENV VECTOR_LOG=WARN
CMD ["sh", "-c", "node ./server.js 2>&1 | vector -vv"]

