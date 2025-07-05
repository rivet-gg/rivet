import { defineConfig } from 'tsup'

export default defineConfig({
    entry: ['src/index.ts'],
    splitting: false,
    minify: true,
    clean: true,
    sourcemap: false
});