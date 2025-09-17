import { createDialogHook, useDialog } from "@/components";

const d = {
	...useDialog,
	CreateNamespace: createDialogHook(
		() => import("@/app/dialogs/create-namespace-frame"),
	),
	CreateProject: createDialogHook(
		() => import("@/app/dialogs/create-project-frame"),
	),
	ConnectVercel: createDialogHook(
		() => import("@/app/dialogs/connect-vercel-frame"),
	),
	ConnectRailway: createDialogHook(
		() => import("@/app/dialogs/connect-railway-frame"),
	),
	Billing: createDialogHook(() => import("@/app/dialogs/billing-frame")),
};

export { d as useDialog };
