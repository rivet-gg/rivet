import {
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "../../ui/dialog";
import { Flex } from "../../ui/flex";
import { useActorsView } from "../actors-view-context-provider";
import * as ActorCreateForm from "../form/actor-create-form";
import type { DialogContentProps } from "../hooks";
import { useMutation } from "@tanstack/react-query";
import { useManagerQueries } from "../manager-queries-context";

interface ContentProps extends DialogContentProps {}

export default function CreateActorDialog({ onClose }: ContentProps) {
	const { mutateAsync } = useMutation(
		useManagerQueries().createActorMutationOptions(),
	);

	const { copy } = useActorsView();

	return (
		<>
			<ActorCreateForm.Form
				onSubmit={async (values) => {
					const key = JSON.parse(values.key);
					await mutateAsync({
						name: values.name,
						input: values.input
							? JSON.parse(values.input)
							: undefined,
						key: Array.isArray(key) ? key : [key],
					});
					onClose?.();
				}}
				defaultValues={{ name: "" }}
			>
				<DialogHeader>
					<DialogTitle>{copy.createActorModal.title}</DialogTitle>
					<DialogDescription>
						{copy.createActorModal.description}
					</DialogDescription>
				</DialogHeader>
				<Flex gap="4" direction="col">
					<ActorCreateForm.Build />
					{/* <ActorCreateForm.Region /> */}
					{/* <ActorCreateForm.Tags /> */}
					<ActorCreateForm.Keys />
					<ActorCreateForm.JsonInput />
				</Flex>
				<DialogFooter>
					<ActorCreateForm.Submit type="submit">
						Create
					</ActorCreateForm.Submit>
				</DialogFooter>
			</ActorCreateForm.Form>
		</>
	);
}
