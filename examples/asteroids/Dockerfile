# syntax=docker/dockerfile:1

# === Builder ===
FROM node:20-alpine AS build
WORKDIR /app/build
COPY package.json yarn.lock tsconfig.json ./
RUN yarn install  --frozen-lockfile
COPY ./server ./server
COPY ./shared ./shared
RUN yarn run build:server
RUN rm -rf ./node_modules

# === Runner ===
FROM node:20-alpine
ENV NODE_ENV=production
WORKDIR /app
COPY --from=build /app/build/package.json /app/build/yarn.lock ./
COPY --from=build /app/build/build/server ./
RUN yarn install  --frozen-lockfile --production
RUN adduser -D server
USER server
CMD ["node", "-r", "module-alias/register", "server/index.js"]
