import { createRoute, z } from "@hono/zod-openapi";
import { ErrorDefaultResponses, BearerAuthSchema } from "./common";

// Schemas
const DisplayNameSchema = z.string().min(1).max(32);
const AccountNumberSchema = z.number().int().min(1).max(9999);
const BioSchema = z.string().max(512);

const IdentityGetProfileResponseSchema = z.object({
    identity: z.object({
        identity_id: z.string().uuid(),
        display_name: DisplayNameSchema,
        account_number: AccountNumberSchema,
        avatar_url: z.string().url().optional(),
        bio: BioSchema.optional(),
        is_registered: z.boolean(),
        is_admin: z.boolean(),
        is_game_linked: z.boolean().optional(),
        dev_state: z.enum(["inactive", "pending", "accepted"]).optional(),
        follower_count: z.number().int().min(0),
        following_count: z.number().int().min(0),
        following: z.boolean(),
        is_following_me: z.boolean(),
        is_mutual_following: z.boolean(),
        join_ts: z.string().datetime(),
        external: z.object({
            profile: z.string().url().optional(),
            avatar_url: z.string().url().optional()
        }).optional(),
        presence: z.object({
            update_ts: z.string().datetime(),
            status: z.enum(["online", "away", "offline"]),
            game_activity: z.object({
                game: z.object({
                    game_id: z.string().uuid(),
                    display_name: z.string(),
                    logo_url: z.string().url().optional(),
                    banner_url: z.string().url().optional()
                }),
                message: z.string(),
                public_metadata: z.record(z.unknown()).optional(),
                mutual_metadata: z.record(z.unknown()).optional()
            }).optional()
        }).optional(),
        games: z.array(z.object({
            game: z.object({
                game_id: z.string().uuid(),
                display_name: z.string(),
                logo_url: z.string().url().optional(),
                banner_url: z.string().url().optional()
            }),
            statistics: z.record(z.unknown()).optional()
        })),
        groups: z.array(z.object({
            group: z.object({
                group_id: z.string().uuid(),
                display_name: z.string(),
                avatar_url: z.string().url().optional(),
                external: z.object({
                    profile: z.string().url().optional(),
                    avatar_url: z.string().url().optional()
                }).optional(),
                is_current_identity_member: z.boolean()
            })
        })),
        linked_accounts: z.array(z.object({
            account_id: z.string().uuid(),
            complete_ts: z.string().datetime().optional()
        }))
    })
});

const UpdateProfileRequestSchema = z.object({
    display_name: DisplayNameSchema.optional(),
    account_number: AccountNumberSchema.optional(),
    bio: BioSchema.optional()
});

const PrepareAvatarUploadRequestSchema = z.object({
    path: z.string(),
    mime: z.string(),
    content_length: z.number().int().min(0)
});

const IdentityPrepareAvatarUploadResponseSchema = z.object({
    upload_id: z.string().uuid(),
    presigned_request: z.object({
        url: z.string().url(),
        headers: z.record(z.string())
    })
});

// Routes
export const identityGetSelfProfileRoute = createRoute({
    method: "get",
    path: "/identity/identities/self/profile",
    tags: ["Identity"],
    summary: "Get current identity profile",
    description: "Fetches the current identity's profile.",
    operationId: "identity_getSelfProfile",
    request: {},
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: IdentityGetProfileResponseSchema
                }
            },
            description: "Returns the current identity's profile."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const identityUpdateProfileRoute = createRoute({
    method: "post",
    path: "/identity/identities/self/profile",
    tags: ["Identity"],
    summary: "Update current identity profile",
    description: "Updates profile of the current identity.",
    operationId: "identity_updateProfile",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: UpdateProfileRequestSchema
                }
            }
        }
    },
    responses: {
        "204": {
            description: "Profile updated successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const identityPrepareAvatarUploadRoute = createRoute({
    method: "post",
    path: "/identity/identities/avatar-upload/prepare",
    tags: ["Identity"],
    summary: "Prepare avatar upload",
    description: "Prepares an avatar image upload. Complete upload with CompleteIdentityAvatarUpload.",
    operationId: "identity_prepareAvatarUpload",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: PrepareAvatarUploadRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: IdentityPrepareAvatarUploadResponseSchema
                }
            },
            description: "Returns upload details."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const identityCompleteAvatarUploadRoute = createRoute({
    method: "post",
    path: "/identity/identities/avatar-upload/{upload_id}/complete",
    tags: ["Identity"],
    summary: "Complete avatar upload",
    description: "Completes an avatar image upload. Must be called after the file upload process completes.",
    operationId: "identity_completeAvatarUpload",
    request: {
        params: z.object({
            upload_id: z.string().uuid()
        })
    },
    responses: {
        "204": {
            description: "Avatar upload completed successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const identityMarkDeletionRoute = createRoute({
    method: "post",
    path: "/identity/identities/self/delete-request",
    tags: ["Identity"],
    summary: "Mark identity for deletion",
    description: "Marks the current identity for deletion.",
    operationId: "identity_markDeletion",
    responses: {
        "204": {
            description: "Identity marked for deletion successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const identityUnmarkDeletionRoute = createRoute({
    method: "delete",
    path: "/identity/identities/self/delete-request",
    tags: ["Identity"],
    summary: "Unmark identity for deletion",
    description: "Unmarks the current identity for deletion.",
    operationId: "identity_unmarkDeletion",
    responses: {
        "204": {
            description: "Identity unmarked for deletion successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});
