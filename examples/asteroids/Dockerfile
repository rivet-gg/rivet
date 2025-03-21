# syntax=docker/dockerfile:1

# === Builder ===
FROM node:22-alpine AS build
WORKDIR /app/build
COPY package.json yarn.lock tsconfig.json actor-core.ts token.secret.ts .yarnrc.yml vendor/ ./
COPY vendor/ vendor/
RUN yarn install
COPY ./server ./server
COPY ./shared ./shared
RUN yarn run build:server
RUN rm -rf ./node_modules

# === Runner ===
FROM node:22-alpine
ENV NODE_ENV=production
WORKDIR /app
COPY ./.env ./
COPY --from=build /app/build/package.json /app/build/yarn.lock ./
COPY --from=build /app/build/vendor/ ./vendor/
COPY --from=build /app/build/build/server ./
RUN yarn install  --production
RUN adduser -D rivet
USER rivet
CMD ["node", "-r", "module-alias/register", "server/index.js"]
