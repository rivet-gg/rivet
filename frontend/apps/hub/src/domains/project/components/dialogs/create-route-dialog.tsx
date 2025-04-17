import * as EditRouteForm from "@/domains/project/forms/route-edit-form";
import type { DialogContentProps } from "@/hooks/use-dialog";
import {
	Button,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	Label,
} from "@rivet-gg/components";
import { useState } from "react";
import { useCreateRouteMutation } from "../../queries";
import { convertStringToId } from "@/lib/utils";

interface ContentProps extends DialogContentProps {
	projectNameId: string;
	environmentNameId: string;
}

export default function EditRouteDialogContent(props: ContentProps) {
	return <Content {...props} />;
}

function Content({ projectNameId, environmentNameId, onClose }: ContentProps) {
	const { mutateAsync } = useCreateRouteMutation();

	const [tagKeys, setTagKeys] = useState<{ label: string; value: string }[]>(
		() => [],
	);

	const [tagValues, setTagValues] = useState<
		{ label: string; value: string }[]
	>(() => []);

	return (
		<EditRouteForm.Form
			defaultValues={{
				routeName: "",
				slug: "",
				tags: [],
				hostname: "",
				path: "",
			}}
			onSubmit={async (values) => {
				const selectorTags = Object.fromEntries(
					values.tags.map(({ key, value }) => [key, value]),
				);

				await mutateAsync({
					projectNameId,
					environmentNameId,
					hostname: values.hostname,
					nameId: values.slug || convertStringToId(values.routeName),
					// remove the * from the end of the path
					path: values.path.endsWith("/*")
						? values.path.slice(0, -2)
						: values.path,
					selectorTags,
					routeSubpaths: values.path.endsWith("/*"),
				});
				onClose?.();
			}}
		>
			<DialogHeader>
				<DialogTitle>Add Route</DialogTitle>
			</DialogHeader>

			<div className="flex gap-2">
				<EditRouteForm.Name className="flex-1" />
				<EditRouteForm.Slug className="flex-1" />
			</div>

			<div className="flex gap-2">
				<EditRouteForm.Hostname />
				<EditRouteForm.Path />
			</div>

			<Label>Selectors</Label>

			<EditRouteForm.Tags
				keys={tagKeys}
				values={tagValues}
				onCreateKeyOption={(option) =>
					setTagKeys((opts) => [
						...opts,
						{ label: option, value: option },
					])
				}
				onCreateValueOption={(option) =>
					setTagValues((opts) => [
						...opts,
						{ label: option, value: option },
					])
				}
			/>

			<DialogFooter>
				<EditRouteForm.Submit>Save</EditRouteForm.Submit>
				<Button type="button" variant="secondary" onClick={onClose}>
					Close
				</Button>
			</DialogFooter>
		</EditRouteForm.Form>
	);
}
