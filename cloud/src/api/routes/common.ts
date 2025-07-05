import { RouteConfig, z } from "@hono/zod-openapi";

export const ErrorBodySchema = z.object({
    // TODO: Fill in with #/components/schemas/ErrorBody schema
});

export const BearerAuthSchema = {
    type: "http",
    scheme: "bearer"
} as const;

export const ErrorDefaultResponses: RouteConfig["responses"] = {
    "400": {
        content: {
            "application/json": {
                schema: ErrorBodySchema,
            }
        },
        description: "Bad Request"
    },
    "401": {
        content: {
            "application/json": {
                schema: ErrorBodySchema,
            }
        },
        description: "Unauthorized"
    },
    "403": {
        content: {
            "application/json": {
                schema: ErrorBodySchema,
            }
        },
        description: "Forbidden"
    },
    "404": {
        content: {
            "application/json": {
                schema: ErrorBodySchema,
            }
        },
        description: "Not Found"
    },
    "408": {
        content: {
            "application/json": {
                schema: ErrorBodySchema,
            }
        },
        description: "Request Timeout"
    },
    "429": {
        content: {
            "application/json": {
                schema: ErrorBodySchema,
            }
        },
        description: "Too Many Requests"
    },
    "500": {
        content: {
            "application/json": {
                schema: ErrorBodySchema,
            }
        },
        description: "Internal Server Error"
    },
}