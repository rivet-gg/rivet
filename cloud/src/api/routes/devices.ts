import { createRoute, z } from "@hono/zod-openapi";
import { ErrorDefaultResponses } from "./common";

// Request/Response schemas
const CloudDevicesPrepareDeviceLinkResponseSchema = z.object({
    device_link_id: z.string().uuid(),
    device_link_token: z.string(),
    device_link_url: z.string().url()
});

const CloudDevicesGetDeviceLinkResponseSchema = z.object({
    cloud_token: z.string().optional()
});

const CloudDevicesCompleteDeviceLinkRequestSchema = z.object({
    device_link_token: z.string(),
    game_id: z.string().uuid()
});

// Routes
export const cloudDevicesLinksPrepareRoute = createRoute({
    method: "post",
    path: "/cloud/devices/links",
    tags: ["CloudDevicesLinks"],
    summary: "Prepare device link",
    description: "Prepares a device link for connecting a device to the cloud.",
    operationId: "cloud_devices_links_prepare",
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudDevicesPrepareDeviceLinkResponseSchema
                }
            },
            description: "Returns device link preparation details."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudDevicesLinksGetRoute = createRoute({
    method: "get",
    path: "/cloud/devices/links",
    tags: ["CloudDevicesLinks"],
    summary: "Get device link",
    description: "Gets device link information and status.",
    operationId: "cloud_devices_links_get",
    request: {
        query: z.object({
            device_link_token: z.string()
        })
    },
    responses: {
        "200": {
            content: {
                "application/json": {
                    schema: CloudDevicesGetDeviceLinkResponseSchema
                }
            },
            description: "Returns device link information."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});

export const cloudDevicesLinksCompleteRoute = createRoute({
    method: "post",
    path: "/cloud/devices/links/complete",
    tags: ["CloudDevicesLinks"],
    summary: "Complete device link",
    description: "Completes a device link connection.",
    operationId: "cloud_devices_links_complete",
    request: {
        body: {
            content: {
                "application/json": {
                    schema: CloudDevicesCompleteDeviceLinkRequestSchema
                }
            }
        }
    },
    responses: {
        "204": {
            description: "Device link completed successfully."
        },
        ...ErrorDefaultResponses
    },
    security: [{ Bearer: [] }]
});