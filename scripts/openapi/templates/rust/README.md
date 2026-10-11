This override is based on OpenAPI Generator v7.14.0's `rust/reqwest/api.mustache`:
https://github.com/OpenAPITools/openapi-generator/blob/v7.14.0/modules/openapi-generator/src/main/resources/rust/reqwest/api.mustache

The upstream template is licensed under Apache-2.0. The sole customization sets
`Content-Type: application/octet-stream` when sending a binary request body.
`gen_rust.ts` maps the binary schema to `Vec<u8>` so callers supply exact bytes.
