import { createDialogHook, useDialog } from "@/components";

const d = useDialog as typeof useDialog &
	Record<string, ReturnType<typeof createDialogHook>>;
d.CreateNamespace = createDialogHook(
	() => import("@/app/dialogs/create-namespace-frame"),
);

d.CreateProject = createDialogHook(
	() => import("@/app/dialogs/create-project-frame"),
);

d.ConnectVercel = createDialogHook(
	() => import("@/app/dialogs/connect-vercel-frame"),
);

d.ConnectRailway = createDialogHook(
	() => import("@/app/dialogs/connect-railway-frame"),
);

export { d as useDialog };
