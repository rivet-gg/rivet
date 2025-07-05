import { createRoute, z } from "@hono/zod-openapi";
import { ErrorDefaultResponses, BearerAuthSchema } from "./common";

// Shared schemas
const GroupSchema = z.object({
    group_id: z.string().uuid(),
    display_name: z.string(),
    avatar_url: z.string().url().optional(),
    external: z.object({
        profile: z.string().url().optional(),
        avatar_url: z.string().url().optional()
    }).optional(),
    is_current_identity_member: z.boolean(),
    publicity: z.enum(["open", "closed"]).optional(),
    member_count: z.number().int().min(0).optional(),
    owner_identity_id: z.string().uuid().optional(),
    description: z.string().optional(),
    is_developer: z.boolean().optional()
});

const GroupMemberSchema = z.object({
    identity: z.object({
        identity_id: z.string().uuid(),
        display_name: z.string(),
        account_number: z.number().int().min(1).max(9999),
        avatar_url: z.string().url().optional(),
        is_registered: z.boolean(),
        external: z.object({
            profile: z.string().url().optional(),
            avatar_url: z.string().url().optional()
        }).optional(),
        is_admin: z.boolean(),
        is_game_linked: z.boolean().optional(),
        dev_state: z.enum(["inactive", "pending", "accepted"]).optional(),
        follower_count: z.number().int().min(0),
        following_count: z.number().int().min(0),
        following: z.boolean(),
        is_following_me: z.boolean(),
        is_mutual_following: z.boolean(),
        join_ts: z.string().datetime(),
        bio: z.string().optional(),
        linked_accounts: z.array(z.object({
            account_id: z.string().uuid(),
            complete_ts: z.string().datetime().optional()
        }))
    })
});

// Request/Response schemas
const GroupGetProfileResponseSchema = z.object({
    group: GroupSchema
});

const GroupUpdateProfileRequestSchema = z.object({
    display_name: z.string().min(1).max(32).optional(),
    bio: z.string().max(512).optional(),
    publicity: z.enum(["open", "closed"]).optional()
});

const GroupPrepareAvatarUploadRequestSchema = z.object({
    path: z.string(),
    mime: z.string(),
    content_length: z.number().int().min(0)
});

const GroupPrepareAvatarUploadResponseSchema = z.object({
    upload_id: z.string().uuid(),
    presigned_request: z.object({
        url: z.string().url(),
        headers: z.record(z.string())
    })
});

const GroupTransferOwnershipRequestSchema = z.object({
    new_owner_identity_id: z.string().uuid()
});

const GroupListSuggestedResponseSchema = z.object({
    groups: z.array(GroupSchema)
});

const GroupCreateRequestSchema = z.object({
    display_name: z.string().min(1).max(32)
});

const GroupCreateResponseSchema = z.object({
    group_id: z.string().uuid()
});

const GroupCreateInviteRequestSchema = z.object({
    ttl: z.number().int().min(0).optional(),
    use_count: z.number().int().min(0).optional()
});

const GroupCreateInviteResponseSchema = z.object({
    code: z.string()
});

const GroupConsumeInviteResponseSchema = z.object({
    group_id: z.string().uuid()
});

const GroupGetMembersResponseSchema = z.object({
    members: z.array(GroupMemberSchema),
    anchor: z.string().optional()
});

const GroupGetInviteResponseSchema = z.object({
    group: GroupSchema
});

// Routes
export const groupGetProfileRoute = createRoute({
    method: "get",
    path: "/group/groups/{group_id}/profile",
    tags: ["Group"],
    summary: "Get group profile",
    description: "Returns a group profile.",
    operationId: "group_getProfile",
    request: {
        params: z.object({
            group_id: z.string().uuid()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupGetProfileResponseSchema
                }
            },
            description: "Returns the group profile."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupUpdateProfileRoute = createRoute({
    method: "post",
    path: "/group/groups/{group_id}/profile",
    tags: ["Group"],
    summary: "Update group profile",
    description: "Updates a group profile.",
    operationId: "group_updateProfile",
    request: {
        params: z.object({
            group_id: z.string().uuid()
        }),
        body: {
            content: {
                "application/json": {
                    schema: GroupUpdateProfileRequestSchema
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

export const groupPrepareAvatarUploadRoute = createRoute({
    method: "post",
    path: "/group/groups/avatar-upload/prepare",
    tags: ["Group"],
    summary: "Prepare group avatar upload",
    description: "Prepares an avatar image upload. Complete upload with CompleteAvatarUpload.",
    operationId: "group_prepareAvatarUpload",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: GroupPrepareAvatarUploadRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupPrepareAvatarUploadResponseSchema
                }
            },
            description: "Returns upload details."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupCompleteAvatarUploadRoute = createRoute({
    method: "post",
    path: "/group/groups/{group_id}/avatar-upload/{upload_id}/complete",
    tags: ["Group"],
    summary: "Complete group avatar upload",
    description: "Completes an avatar image upload. Must be called after the file upload process completes.",
    operationId: "group_completeAvatarUpload",
    request: {
        params: z.object({
            group_id: z.string().uuid(),
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

export const groupTransferOwnershipRoute = createRoute({
    method: "post",
    path: "/group/groups/{group_id}/transfer-owner",
    tags: ["Group"],
    summary: "Transfer group ownership",
    description: "Transfers ownership of a group to another identity.",
    operationId: "group_transferOwnership",
    request: {
        params: z.object({
            group_id: z.string().uuid()
        }),
        body: {
            content: {
                "application/json": {
                    schema: GroupTransferOwnershipRequestSchema
                }
            }
        }
    },
    responses: {
        "204": {
            description: "Ownership transferred successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupKickMemberRoute = createRoute({
    method: "post",
    path: "/group/groups/{group_id}/kick/{identity_id}",
    tags: ["Group"],
    summary: "Kick group member",
    description: "Kicks an identity from a group. Must be the owner of the group to perform this action.",
    operationId: "group_kickMember",
    request: {
        params: z.object({
            group_id: z.string().uuid(),
            identity_id: z.string().uuid()
        })
    },
    responses: {
        "204": {
            description: "Member kicked successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupBanIdentityRoute = createRoute({
    method: "post",
    path: "/group/groups/{group_id}/bans/{identity_id}",
    tags: ["Group"],
    summary: "Ban identity from group",
    description: "Bans an identity from a group. Must be the owner of the group to perform this action.",
    operationId: "group_banIdentity",
    request: {
        params: z.object({
            group_id: z.string().uuid(),
            identity_id: z.string().uuid()
        })
    },
    responses: {
        "204": {
            description: "Identity banned successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupUnbanIdentityRoute = createRoute({
    method: "delete",
    path: "/group/groups/{group_id}/bans/{identity_id}",
    tags: ["Group"],
    summary: "Unban identity from group",
    description: "Unbans an identity from a group. Must be the owner of the group to perform this action.",
    operationId: "group_unbanIdentity",
    request: {
        params: z.object({
            group_id: z.string().uuid(),
            identity_id: z.string().uuid()
        })
    },
    responses: {
        "204": {
            description: "Identity unbanned successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupListSuggestedRoute = createRoute({
    method: "get",
    path: "/group/groups",
    tags: ["Group"],
    summary: "List suggested groups",
    description: "Returns a list of suggested groups.",
    operationId: "group_listSuggested",
    request: {},
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupListSuggestedResponseSchema
                }
            },
            description: "Returns suggested groups."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupCreateRoute = createRoute({
    method: "post",
    path: "/group/groups",
    tags: ["Group"],
    summary: "Create group",
    description: "Creates a new group.",
    operationId: "group_create",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: GroupCreateRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupCreateResponseSchema
                }
            },
            description: "Returns the created group ID."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupCreateInviteRoute = createRoute({
    method: "post",
    path: "/group/groups/{group_id}/invites",
    tags: ["GroupInvites"],
    summary: "Create group invite",
    description: "Creates a group invite. Can be shared with other identities to let them join this group.",
    operationId: "group_invites_createInvite",
    request: {
        params: z.object({
            group_id: z.string().uuid()
        }),
        body: {
            content: {
                "application/json": {
                    schema: GroupCreateInviteRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupCreateInviteResponseSchema
                }
            },
            description: "Returns the invite code."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupConsumeInviteRoute = createRoute({
    method: "post",
    path: "/group/invites/{group_invite_code}/consume",
    tags: ["GroupInvites"],
    summary: "Consume group invite",
    description: "Consumes a group invite to join a group.",
    operationId: "group_invites_consumeInvite",
    request: {
        params: z.object({
            group_invite_code: z.string()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupConsumeInviteResponseSchema
                }
            },
            description: "Returns the joined group ID."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupLeaveRoute = createRoute({
    method: "post",
    path: "/group/groups/{group_id}/leave",
    tags: ["Group"],
    summary: "Leave group",
    description: "Leaves a group.",
    operationId: "group_leave",
    request: {
        params: z.object({
            group_id: z.string().uuid()
        })
    },
    responses: {
        "204": {
            description: "Left group successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupGetMembersRoute = createRoute({
    method: "get",
    path: "/group/groups/{group_id}/members",
    tags: ["Group"],
    summary: "Get group members",
    description: "Returns a group's members.",
    operationId: "group_getMembers",
    request: {
        params: z.object({
            group_id: z.string().uuid()
        }),
        query: z.object({
            anchor: z.string().optional(),
            count: z.number().optional(),
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupGetMembersResponseSchema
                }
            },
            description: "Returns group members."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const groupGetInviteRoute = createRoute({
    method: "get",
    path: "/group/invites/{group_invite_code}",
    tags: ["GroupInvites"],
    summary: "Get group invite",
    description: "Inspects a group invite returning information about the team that created it.",
    operationId: "group_invites_getInvite",
    request: {
        params: z.object({
            group_invite_code: z.string()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GroupGetInviteResponseSchema
                }
            },
            description: "Returns invite information."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});
