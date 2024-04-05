# Generating

## Step 1: Cloud & build Fern

```sh
gh repo clone rivet-gg/fern
cd fern
yarn install
yarn husky install
yarn dist:cli:dev
```

## Step 2: Generate

In the Rivet repo:

```
FERN_REPO_PATH=/path/to/fern ./scripts/fern/gen.sh
```
