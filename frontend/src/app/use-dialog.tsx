import { createDialogHook, useDialog } from "@/components";

const d = useDialog as typeof useDialog &
	Record<string, ReturnType<typeof createDialogHook>>;
d.CreateNamespace = createDialogHook(
	() => import("@/app/dialogs/create-namespace-frame"),
);

d.CreateProject = createDialogHook(
	() => import("@/app/dialogs/create-project-frame"),
);

export { d as useDialog };
