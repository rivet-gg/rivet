import * as EditRouteForm from "@/domains/project/forms/route-edit-form";
import type { DialogContentProps } from "@/hooks/use-dialog";
import {
	Button,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	Label,
} from "@rivet-gg/components";
import { useSuspenseQuery } from "@tanstack/react-query";
import { useState } from "react";
import { routeQueryOptions, usePatchRouteMutation } from "../../queries";
import type { Rivet } from "@rivet-gg/api";

interface OptionalContentProps extends DialogContentProps {
	projectNameId: string;
	environmentNameId: string;
	routeNameId?: string;
}

export default function EditRouteDialogContent(props: OptionalContentProps) {
	if (!props.routeNameId) {
		return null;
	}

	return <GuardedContent {...props} routeNameId={props.routeNameId} />;
}

function GuardedContent({
	routeNameId,
	projectNameId,
	environmentNameId,
	onClose,
}: Omit<OptionalContentProps, "routeNameId"> & { routeNameId: string }) {
	const { data } = useSuspenseQuery(
		routeQueryOptions({
			routeNameId,
			projectNameId,
			environmentNameId,
		}),
	);

	if (!data) {
		return null;
	}

	return (
		<Content
			{...data}
			projectNameId={projectNameId}
			environmentNameId={environmentNameId}
			onClose={onClose}
		/>
	);
}

interface ContentProps extends DialogContentProps, Rivet.routes.Route {
	projectNameId: string;
	environmentNameId: string;
}

function Content({
	projectNameId,
	environmentNameId,
	nameId,
	hostname,
	routeSubpaths,
	path,
	selectorTags,
	onClose,
}: ContentProps) {
	const { mutateAsync } = usePatchRouteMutation();

	const tags = Object.entries(selectorTags || {}).map(([key, value]) => ({
		key,
		value,
	}));

	const [tagKeys, setTagKeys] = useState<{ label: string; value: string }[]>(
		() => tags.map(({ key }) => ({ label: key, value: key })),
	);

	const [tagValues, setTagValues] = useState<
		{ label: string; value: string }[]
	>(() => tags.map(({ value }) => ({ label: value, value })));

	return (
		<EditRouteForm.Form
			defaultValues={{
				routeName: nameId,
				slug: nameId,
				tags: Object.entries(selectorTags || {}).map(
					([key, value]) => ({
						key,
						value,
					}),
				),
				hostname,
				path: routeSubpaths ? `${path}/*` : path,
			}}
			onSubmit={async (values) => {
				const selectorTags = Object.fromEntries(
					values.tags.map(({ key, value }) => [key, value]),
				);

				await mutateAsync({
					projectNameId,
					environmentNameId,
					routeNameId: nameId,
					hostname: values.hostname,
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
				<DialogTitle>Edit Route</DialogTitle>
			</DialogHeader>

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
