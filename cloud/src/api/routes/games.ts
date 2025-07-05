import { createRoute, z } from "@hono/zod-openapi";
import { ErrorDefaultResponses, BearerAuthSchema } from "./common";

// Shared schemas
const GameSchema = z.object({
    game_id: z.string().uuid(),
    name_id: z.string(),
    display_name: z.string(),
    description: z.string().optional(),
    logo_url: z.string().url().optional(),
    banner_url: z.string().url().optional(),
    url: z.string().url().optional(),
    developer: z.object({
        group_id: z.string().uuid(),
        display_name: z.string(),
        avatar_url: z.string().url().optional(),
        external: z.object({
            profile: z.string().url().optional(),
            avatar_url: z.string().url().optional()
        }).optional(),
        is_current_identity_member: z.boolean()
    }),
    total_player_count: z.number().int().min(0)
});

const GameNamespaceSchema = z.object({
    namespace_id: z.string().uuid(),
    display_name: z.string(),
    name_id: z.string(),
    version: z.object({
        version_id: z.string().uuid(),
        display_name: z.string(),
        create_ts: z.string().datetime()
    }),
    config: z.object({
        cdn: z.object({
            build_command: z.string().optional(),
            build_output: z.string().optional(),
            site_id: z.string().optional()
        }).optional(),
        matchmaker: z.object({
            game_modes: z.array(z.object({
                name_id: z.string(),
                regions: z.array(z.object({
                    region_id: z.string().uuid(),
                    tier_name_id: z.string(),
                    idle_lobbies: z.object({
                        min_idle_lobbies: z.number().int().min(0),
                        max_idle_lobbies: z.number().int().min(0)
                    })
                })),
                max_players: z.number().int().min(1),
                max_players_direct: z.number().int().min(1),
                max_players_party: z.number().int().min(1),
                docker: z.object({
                    image_id: z.string().uuid(),
                    ports: z.record(z.object({
                        port: z.number().int().min(1).max(65535),
                        port_range: z.object({
                            min: z.number().int().min(1).max(65535),
                            max: z.number().int().min(1).max(65535)
                        }).optional(),
                        protocol: z.enum(["http", "https", "tcp", "tcp_tls", "udp"])
                    })),
                    env: z.record(z.string()),
                    network_mode: z.enum(["bridge", "host"]).optional(),
                    resources: z.object({
                        cpu: z.number().int().min(0),
                        memory: z.number().int().min(0)
                    })
                }).optional(),
                actions: z.object({
                    create: z.object({
                        enabled: z.boolean().optional()
                    }).optional(),
                    find: z.object({
                        enabled: z.boolean().optional()
                    }).optional(),
                    join: z.object({
                        enabled: z.boolean().optional()
                    }).optional()
                }).optional(),
                tier: z.string().optional()
            }))
        }).optional(),
        kv: z.object({}).optional(),
        identity: z.object({
            custom_display_names: z.array(z.object({
                display_name: z.string()
            })).optional(),
            custom_avatars: z.array(z.object({
                upload_id: z.string().uuid()
            })).optional()
        }).optional()
    })
});

const BuildSchema = z.object({
    build_id: z.string().uuid(),
    upload_id: z.string().uuid(),
    display_name: z.string(),
    create_ts: z.string().datetime(),
    content_length: z.number().int().min(0),
    tags: z.record(z.string()).optional(),
    complete: z.boolean()
});

const ServerSchema = z.object({
    server_id: z.string().uuid(),
    environment: z.string(),
    datacenter: z.object({
        datacenter_id: z.string().uuid(),
        display_name: z.string(),
        name_id: z.string()
    }),
    tags: z.record(z.string()).optional(),
    runtime: z.object({
        build: z.string().uuid(),
        arguments: z.array(z.string()),
        environment: z.record(z.string())
    }),
    network: z.object({
        mode: z.enum(["bridge", "host"]),
        ports: z.record(z.object({
            port: z.number().int().min(1).max(65535),
            port_range: z.object({
                min: z.number().int().min(1).max(65535),
                max: z.number().int().min(1).max(65535)
            }).optional(),
            protocol: z.enum(["http", "https", "tcp", "tcp_tls", "udp"])
        }))
    }),
    resources: z.object({
        cpu: z.number().int().min(0),
        memory: z.number().int().min(0)
    }),
    lifecycle: z.object({
        kill_timeout: z.number().int().min(0).optional()
    }).optional(),
    create_ts: z.string().datetime(),
    start_ts: z.string().datetime().optional(),
    destroy_ts: z.string().datetime().optional()
});

// Request/Response schemas
const CloudGamesGetGamesResponseSchema = z.object({
    games: z.array(GameSchema),
    groups: z.array(z.object({
        group_id: z.string().uuid(),
        display_name: z.string(),
        avatar_url: z.string().url().optional(),
        external: z.object({
            profile: z.string().url().optional(),
            avatar_url: z.string().url().optional()
        }).optional(),
        is_current_identity_member: z.boolean()
    }))
});

const CloudGamesCreateGameRequestSchema = z.object({
    name_id: z.string().regex(/^[a-z0-9\-_]{3,16}$/),
    display_name: z.string().min(1).max(32),
    developer_group_id: z.string().uuid()
});

const CloudGamesCreateGameResponseSchema = z.object({
    game_id: z.string().uuid()
});

const CloudGamesGetGameByIdResponseSchema = z.object({
    game: GameSchema
});

const CloudGamesCreateCloudTokenResponseSchema = z.object({
    token: z.string()
});

const CloudGamesValidateGameRequestSchema = z.object({
    display_name: z.string().min(1).max(32),
    name_id: z.string().regex(/^[a-z0-9\-_]{3,16}$/)
});

const CloudGamesValidateGameResponseSchema = z.object({
    errors: z.array(z.object({
        path: z.array(z.string()),
        code: z.string(),
        message: z.string()
    }))
});

const PrepareLogoUploadRequestSchema = z.object({
    path: z.string(),
    mime: z.string(),
    content_length: z.number().int().min(0)
});

const PrepareLogoUploadResponseSchema = z.object({
    upload_id: z.string().uuid(),
    presigned_request: z.object({
        url: z.string().url(),
        headers: z.record(z.string())
    })
});

const CloudGamesNamespacesCreateGameNamespaceRequestSchema = z.object({
    namespace_id: z.string().regex(/^[a-z0-9\-_]{3,16}$/),
    display_name: z.string().min(1).max(32),
    version_id: z.string().uuid()
});

const CloudGamesNamespacesCreateGameNamespaceResponseSchema = z.object({
    namespace_id: z.string().uuid()
});

const CloudGamesNamespacesGetGameNamespaceByIdResponseSchema = z.object({
    namespace: GameNamespaceSchema
});

const CloudGamesNamespacesCreateGameNamespaceTokenPublicResponseSchema = z.object({
    token: z.string()
});

const ServersListBuildsResponseSchema = z.object({
    builds: z.array(BuildSchema)
});

const ServersGetBuildResponseSchema = z.object({
    build: BuildSchema
});

const ServersCreateBuildRequestSchema = z.object({
    name: z.string().min(1).max(64),
    image_tag: z.string().optional(),
    image_file: z.object({
        path: z.string(),
        mime: z.string(),
        content_length: z.number().int().min(0)
    }).optional(),
    image_upload_id: z.string().uuid().optional(),
    multipart_upload: z.boolean().optional(),
    kind: z.enum(["docker_image", "oci_bundle"]).optional(),
    compression: z.enum(["none", "lz4"]).optional(),
    tags: z.record(z.string()).optional()
});

const ServersCreateBuildResponseSchema = z.object({
    build_id: z.string().uuid(),
    upload_id: z.string().uuid(),
    image_presigned_request: z.object({
        url: z.string().url(),
        headers: z.record(z.string())
    }).optional(),
    image_presigned_requests: z.array(z.object({
        url: z.string().url(),
        headers: z.record(z.string()),
        content_length: z.number().int().min(0)
    })).optional()
});

const ServersPatchBuildTagsRequestSchema = z.object({
    tags: z.record(z.string()),
    exclusive_tags: z.array(z.string()).optional()
});

const ServersPatchBuildTagsResponseSchema = z.object({
    build: BuildSchema
});

const ServersListDatacentersResponseSchema = z.object({
    datacenters: z.array(z.object({
        datacenter_id: z.string().uuid(),
        display_name: z.string(),
        name_id: z.string()
    }))
});

const ServersListServersResponseSchema = z.object({
    servers: z.array(ServerSchema)
});

const ServersGetServerResponseSchema = z.object({
    server: ServerSchema
});

const ServersGetServerLogsResponseSchema = z.object({
    lines: z.array(z.object({
        line: z.string(),
        timestamp: z.string().datetime(),
        stream: z.enum(["stdout", "stderr"])
    })),
    timestamps: z.array(z.string().datetime())
});

const GamesEnvironmentsCreateServiceTokenResponseSchema = z.object({
    token: z.string()
});

// Main Cloud Games Routes
export const cloudGamesGetGamesRoute = createRoute({
    method: "get",
    path: "/cloud/games",
    tags: ["CloudGames"],
    summary: "List games",
    description: "Returns a list of games where the current identity is a group member of the development team.",
    operationId: "cloud_games_getGames",
    request: {},
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesGetGamesResponseSchema
                }
            },
            description: "Returns list of games."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesCreateGameRoute = createRoute({
    method: "post",
    path: "/cloud/games",
    tags: ["CloudGames"],
    summary: "Create game",
    description: "Creates a new game.",
    operationId: "cloud_games_createGame",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: CloudGamesCreateGameRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesCreateGameResponseSchema
                }
            },
            description: "Returns the created game ID."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesValidateGameRoute = createRoute({
    method: "post",
    path: "/cloud/games/validate",
    tags: ["CloudGames"],
    summary: "Validate game",
    description: "Validates information used to create a new game.",
    operationId: "cloud_games_validateGame",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: CloudGamesValidateGameRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesValidateGameResponseSchema
                }
            },
            description: "Returns validation errors."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesGetGameByIdRoute = createRoute({
    method: "get",
    path: "/cloud/games/{game_id}",
    tags: ["CloudGames"],
    summary: "Get game by ID",
    description: "Returns a game by its game ID.",
    operationId: "cloud_games_getGameById",
    request: {
        params: z.object({
            game_id: z.string().uuid()
        }),
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesGetGameByIdResponseSchema
                }
            },
            description: "Returns the game."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesCreateCloudTokenRoute = createRoute({
    method: "post",
    path: "/cloud/games/{game_id}/tokens/cloud",
    tags: ["CloudGames"],
    summary: "Create cloud token",
    description: "Creates a new game cloud token.",
    operationId: "cloud_games_tokens_createCloudToken",
    request: {
        params: z.object({
            game_id: z.string().uuid()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesCreateCloudTokenResponseSchema
                }
            },
            description: "Returns the cloud token."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesPrepareLogoUploadRoute = createRoute({
    method: "post",
    path: "/cloud/games/{game_id}/logo-upload/prepare",
    tags: ["CloudGames"],
    summary: "Prepare logo upload",
    description: "Prepares a game logo upload.",
    operationId: "cloud_games_gameLogoUploadPrepare",
    request: {
        params: z.object({
            game_id: z.string().uuid()
        }),
        body: {
            content: {
                "application/json": {
                    schema: PrepareLogoUploadRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: PrepareLogoUploadResponseSchema
                }
            },
            description: "Returns upload details."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesCompleteLogoUploadRoute = createRoute({
    method: "post",
    path: "/cloud/games/{game_id}/logo-upload/{upload_id}/complete",
    tags: ["CloudGames"],
    summary: "Complete logo upload",
    description: "Completes a game logo upload.",
    operationId: "cloud_games_gameLogoUploadComplete",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            upload_id: z.string().uuid()
        })
    },
    responses: {
        "204": {
            description: "Logo upload completed successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

// Namespaces Routes
export const cloudGamesNamespacesCreateGameNamespaceRoute = createRoute({
    method: "post",
    path: "/cloud/games/{game_id}/namespaces",
    tags: ["CloudGamesNamespaces"],
    summary: "Create namespace",
    description: "Creates a new namespace for the given game.",
    operationId: "cloud_games_namespaces_createGameNamespace",
    request: {
        params: z.object({
            game_id: z.string().uuid()
        }),
        body: {
            content: {
                "application/json": {
                    schema: CloudGamesNamespacesCreateGameNamespaceRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesNamespacesCreateGameNamespaceResponseSchema
                }
            },
            description: "Returns the created namespace ID."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesNamespacesGetGameNamespaceByIdRoute = createRoute({
    method: "get",
    path: "/cloud/games/{game_id}/namespaces/{namespace_id}",
    tags: ["CloudGamesNamespaces"],
    summary: "Get namespace by ID",
    description: "Gets a game namespace by namespace ID.",
    operationId: "cloud_games_namespaces_getGameNamespaceById",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            namespace_id: z.string().uuid()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesNamespacesGetGameNamespaceByIdResponseSchema
                }
            },
            description: "Returns the namespace."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudGamesNamespacesCreateGameNamespaceTokenPublicRoute = createRoute({
    method: "post",
    path: "/cloud/games/{game_id}/namespaces/{namespace_id}/tokens/public",
    tags: ["CloudGamesNamespaces"],
    summary: "Create public token",
    description: "Creates a public token for the given namespace.",
    operationId: "cloud_games_namespaces_createGameNamespaceTokenPublic",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            namespace_id: z.string().uuid()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudGamesNamespacesCreateGameNamespaceTokenPublicResponseSchema
                }
            },
            description: "Returns the public token."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

// Servers/Builds Routes
export const serversListBuildsRoute = createRoute({
    method: "get",
    path: "/games/{game_id}/environments/{environment_id}/builds",
    tags: ["Servers"],
    summary: "List builds",
    description: "Lists all builds of the game associated with the token used.",
    operationId: "servers_builds_list",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid(),
        }),
        query: z.object({
            tags_json: z.string().optional()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersListBuildsResponseSchema
                }
            },
            description: "Returns list of builds."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversGetBuildRoute = createRoute({
    method: "get",
    path: "/games/{game_id}/environments/{environment_id}/builds/{build_id}",
    tags: ["Servers"],
    summary: "Get build",
    description: "Gets a specific build.",
    operationId: "servers_builds_get",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid(),
            build_id: z.string().uuid()
        }),
        query: z.object({
            tags_json: z.string().optional()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersGetBuildResponseSchema
                }
            },
            description: "Returns the build."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversPrepareBuildRoute = createRoute({
    method: "post",
    path: "/games/{game_id}/environments/{environment_id}/builds/prepare",
    tags: ["Servers"],
    summary: "Prepare build",
    description: "Creates a new game build for the given game.",
    operationId: "servers_builds_prepare",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid()
        }),
        body: {
            content: {
                "application/json": {
                    schema: ServersCreateBuildRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersCreateBuildResponseSchema
                }
            },
            description: "Returns build preparation details."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversCompleteBuildRoute = createRoute({
    method: "post",
    path: "/games/{game_id}/environments/{environment_id}/builds/{build_id}/complete",
    tags: ["Servers"],
    summary: "Complete build",
    description: "Marks an upload as complete.",
    operationId: "servers_builds_complete",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid(),
            build_id: z.string().uuid()
        })
    },
    responses: {
        "204": {
            description: "Build completed successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversPatchBuildTagsRoute = createRoute({
    method: "patch",
    path: "/games/{game_id}/environments/{environment_id}/builds/{build_id}/tags",
    tags: ["Servers"],
    summary: "Update build tags",
    description: "Updates build tags.",
    operationId: "servers_builds_patchTags",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid(),
            build_id: z.string().uuid()
        }),
        body: {
            content: {
                "application/json": {
                    schema: ServersPatchBuildTagsRequestSchema
                }
            }
        }
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersPatchBuildTagsResponseSchema
                }
            },
            description: "Returns updated build."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversListDatacentersRoute = createRoute({
    method: "get",
    path: "/games/{game_id}/environments/{environment_id}/datacenters",
    tags: ["Servers"],
    summary: "List datacenters",
    description: "Lists available datacenters.",
    operationId: "servers_datacenters_list",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersListDatacentersResponseSchema
                }
            },
            description: "Returns list of datacenters."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversListServersRoute = createRoute({
    method: "get",
    path: "/games/{game_id}/environments/{environment_id}/servers",
    tags: ["Servers"],
    summary: "List servers",
    description: "Lists all servers associated with the token used.",
    operationId: "servers_list",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid()
        }),
        query: z.object({
            tags_json: z.string().optional(),
            include_destroyed: z.boolean().optional(),
            cursor: z.string().optional()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersListServersResponseSchema
                }
            },
            description: "Returns list of servers."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversGetServerRoute = createRoute({
    method: "get",
    path: "/games/{game_id}/environments/{environment_id}/servers/{server_id}",
    tags: ["Servers"],
    summary: "Get server",
    description: "Gets a dynamic server.",
    operationId: "servers_get",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid(),
            server_id: z.string().uuid()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersGetServerResponseSchema
                }
            },
            description: "Returns the server."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const serversGetServerLogsRoute = createRoute({
    method: "get",
    path: "/games/{game_id}/environments/{environment_id}/servers/{server_id}/logs",
    tags: ["Servers"],
    summary: "Get server logs",
    description: "Returns the logs for a given server.",
    operationId: "servers_logs_get",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid(),
            server_id: z.string().uuid()
        }),
        query: z.object({
            stream: z.enum(["stdout", "stderr"]),
            // TODO: is thi snecessary
            // watch_index: z.string().optional()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: ServersGetServerLogsResponseSchema
                }
            },
            description: "Returns server logs."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const gamesEnvironmentsCreateServiceTokenRoute = createRoute({
    method: "post",
    path: "/games/{game_id}/environments/{environment_id}/tokens/service",
    tags: ["GamesEnvironments"],
    summary: "Create service token",
    description: "Creates a new environment service token.",
    operationId: "games_environments_tokens_createServiceToken",
    request: {
        params: z.object({
            game_id: z.string().uuid(),
            environment_id: z.string().uuid()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: GamesEnvironmentsCreateServiceTokenResponseSchema
                }
            },
            description: "Returns the service token."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});