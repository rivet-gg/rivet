import { createRoute, z } from "@hono/zod-openapi";
import { ErrorDefaultResponses, BearerAuthSchema } from "./common";

// Shared schemas
const CaptchaConfigSchema = z
    .object({
        turnstile: z.object({
                client_response: z.string()
            }).openapi({
                description: "Captcha configuration."
            }).optional(),
        hcaptcha: z
            .object({
                client_response: z.string()
            }).openapi({
                description: "Captcha configuration."
            }).optional()
    }).openapi({
        type: "object",
        description: "Methods to verify a captcha",
    })

const AuthCompleteStatusSchema = z
    .enum([
        "switch_identity",
        "linked_account_added", 
        "already_complete",
        "expired",
        "too_many_attempts",
        "incorrect"
    ])
    .openapi({
        description: "Represents the state of an external account linking process."
    });

// Request/Response schemas
const AuthIdentityStartEmailVerificationRequestSchema = z.object({
    email: z.string().email(),
    captcha: CaptchaConfigSchema.optional(),
    game_id: z.string().uuid().optional()
});

const AuthIdentityStartEmailVerificationResponseSchema = z.object({
    verification_id: z.string().uuid()
});

const AuthIdentityCompleteEmailVerificationRequestSchema = z.object({
    verification_id: z.string().uuid(),
    code: z
        .string()
        .openapi({
            description: "The code sent to the requestee's email."
        })
});

const AuthIdentityCompleteEmailVerificationResponseSchema = z.object({
    status: AuthCompleteStatusSchema
});

const AuthRefreshIdentityTokenRequestSchema = z.object({
    logout: z
        .boolean()
        .optional()
        .openapi({
            description: "When `true`, the current identity for the provided cookie will be logged out and a new identity will be returned."
        })
});

const AuthRefreshIdentityTokenResponseSchema = z.object({
    token: z
        .string()
        .openapi({
            description: "A JSON Web Token. Slightly modified to include a description prefix and use Protobufs of JSON."
        }),
    exp: z.string(),
    identity_id: z.string().uuid()
});

const CloudAuthAgentIdentitySchema = z.object({
    identity_id: z.string().uuid()
});

const CloudAuthAgentGameCloudSchema = z.object({
    game_id: z.string().uuid()
});

const CloudAuthAgentSchema = z.object({
    identity: CloudAuthAgentIdentitySchema.optional(),
    game_cloud: CloudAuthAgentGameCloudSchema.optional()
});

const CloudInspectResponseSchema = z.object({
    agent: CloudAuthAgentSchema
});

// Routes
export const authIdentityEmailStartVerificationRoute = createRoute({
    method: "post",
    path: "/auth/identity/email/start-verification",
    tags: ["AuthIdentityEmail"],
    summary: "Start email verification",
    description: "Starts the verification process for linking an email to your identity.",
    operationId: "auth_identity_email_startEmailVerification",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: AuthIdentityStartEmailVerificationRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: AuthIdentityStartEmailVerificationResponseSchema
                }
            },
            description: "Returns verification ID."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const authIdentityEmailCompleteVerificationRoute = createRoute({
    method: "post",
    path: "/auth/identity/email/complete-verification",
    tags: ["AuthIdentityEmail"],
    summary: "Complete email verification",
    description: "Completes the email verification process.",
    operationId: "auth_identity_email_completeEmailVerification",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: AuthIdentityCompleteEmailVerificationRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: AuthIdentityCompleteEmailVerificationResponseSchema
                }
            },
            description: "Returns verification completion status."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const authTokensRefreshIdentityTokenRoute = createRoute({
    method: "post",
    path: "/auth/tokens/identity",
    tags: ["AuthTokens"],
    summary: "Refresh identity token",
    description: "Refreshes the current identity's token and sets authentication headers.",
    operationId: "auth_tokens_refreshIdentityToken",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: AuthRefreshIdentityTokenRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: AuthRefreshIdentityTokenResponseSchema
                }
            },
            description: "Returns refreshed token information."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudAuthInspectRoute = createRoute({
    method: "get",
    path: "/cloud/auth/inspect",
    tags: ["CloudAuth"],
    summary: "Inspect authentication",
    description: "Returns information about the current authenticated agent.",
    operationId: "cloud_auth_inspect",
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudInspectResponseSchema
                }
            },
            description: "Returns authentication agent information."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});