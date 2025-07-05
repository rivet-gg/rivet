import { createRoute, z } from "@hono/zod-openapi";
import { ErrorDefaultResponses } from "./common";

const CloudBootstrapClusterSchema = z
    .enum([
        "enterprise",
        "oss"
    ])
    .openapi({
        description: "The type of cluster that the cloud is currently running."
    });

const CloudBootstrapAccessSchema = z.enum([
    "public",
    "private",
    "development"
]);

const CloudBootstrapDomainsSchema = z
    .object({
        cdn: z.string().optional(),
        job: z.string().optional(),
        api: z.string().optional(),
        opengb: z.string().optional(),
    })
    .openapi({
        description: "Domains that host parts of Rivet"
    });

const CloudBootstrapOriginsSchema = z
    .object({
        hub: z.string()
    })
    .openapi({
        description: "Origins used to build URLs from"
    });

const CloudBootstrapCaptchaTurnstileSchema = z.object({
    site_key: z.string()
});

const CloudBootstrapCaptchaSchema = z.object({
    turnstile: CloudBootstrapCaptchaTurnstileSchema.optional()
});

const CloudBootstrapLoginMethodsSchema = z.object({
    email: z.boolean(),
    access_token: z.boolean().optional(),
});

const CloudBootstrapResponseSchema = z.object({
    cluster: CloudBootstrapClusterSchema,
    access: CloudBootstrapAccessSchema,
    domains: CloudBootstrapDomainsSchema,
    origins: CloudBootstrapOriginsSchema,
    captcha: CloudBootstrapCaptchaSchema,
    login_methods: CloudBootstrapLoginMethodsSchema,
    deploy_hash: z.string()
});

export const cloudBootstrapGetRoute = createRoute({
    method: "get",
    path: "/cloud/bootstrap",
    tags: ["CloudBootstrap"],
    summary: "Returns the basic information required to use the cloud APIs.",
    operationId: "cloud_bootstrap",
    responses: {
        "200": {
            content: {
                'application/json': {
                    schema: CloudBootstrapResponseSchema,
                },
            },
            description: "Returns the basic information required to use the cloud APIs.",
        },
        ...ErrorDefaultResponses
    }
});

// app.openAPIRegistry.registerComponent('securitySchemes', 'Bearer', {
//   type: 'http',
//   scheme: 'bearer',
// })

export * from './auth';
export * from './devices';
export * from './games';
export * from './groups';
export * from './identity';