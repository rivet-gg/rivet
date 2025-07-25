import { type UseFormReturn, useFormContext } from "react-hook-form";
import z from "zod";

import { BuildSelect } from "../build-select";
import { createSchemaForm } from "../../lib/create-schema-form";
import {
	FormControl,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "../../ui/form";
import { JsonCode } from "../../code-mirror";

const jsonValid = z.custom<string>((value) => {
	try {
		JSON.parse(value);
		return true;
	} catch {
		return false;
	}
}, "Must be valid JSON");

export const formSchema = z.object({
	name: z.string().nonempty("Build is required"),
	// regionId: z.string(),
	key: jsonValid
		.refine((val) => {
			if (Array.isArray(JSON.parse(val))) {
				return true;
			}
			if (typeof JSON.parse(val) === "string") {
				return true;
			}
			return false;
		}, "Must be a JSON array or a string")
		.refine((val) => {
			const parsed = JSON.parse(val);
			if (Array.isArray(parsed)) {
				return parsed.every((item) => typeof item === "string");
			}
			return true;
		}, "All items in the array must be strings"),
	input: jsonValid.optional(),
	// tags: tagsFormSchema.shape.tags,
});

export type FormValues = z.infer<typeof formSchema>;
export type SubmitHandler = (
	values: FormValues,
	form: UseFormReturn<FormValues>,
) => Promise<void>;

const { Form, Submit } = createSchemaForm(formSchema);
export { Form, Submit };

export const Build = () => {
	const { control } = useFormContext<FormValues>();
	return (
		<FormField
			control={control}
			name="name"
			render={({ field }) => (
				<FormItem>
					<FormLabel>Definition</FormLabel>
					<FormControl>
						<BuildSelect
							onValueChange={field.onChange}
							value={field.value}
						/>
					</FormControl>
					<FormDescription>
						Used to differentiate between different actor types.
						Corresponds to the bla bla bal bal.
					</FormDescription>
					<FormMessage />
				</FormItem>
			)}
		/>
	);
};

// export const Region = () => {
// 	const { control } = useFormContext<FormValues>();

// 	return (
// 		<FormField
// 			control={control}
// 			name="regionId"
// 			render={({ field }) => (
// 				<FormItem>
// 					<FormLabel>Region</FormLabel>
// 					<FormControl>
// 						<RegionSelect
// 							onValueChange={field.onChange}
// 							value={field.value}
// 						/>
// 					</FormControl>
// 					<FormMessage />
// 				</FormItem>
// 			)}
// 		/>
// 	);
// };

export const Keys = () => {
	const { control } = useFormContext<FormValues>();
	return (
		<FormField
			control={control}
			name="key"
			render={({ field }) => (
				<FormItem>
					<FormLabel>Key</FormLabel>
					<FormControl>
						<JsonCode
							onChange={field.onChange}
							value={field.value}
						/>
					</FormControl>
					<FormDescription>
						Either a JSON array or a string.
					</FormDescription>
					<FormMessage />
				</FormItem>
			)}
		/>
	);
};

export const JsonInput = () => {
	const { control } = useFormContext<FormValues>();
	return (
		<FormField
			control={control}
			name="input"
			render={({ field }) => (
				<FormItem>
					<FormLabel>Input</FormLabel>
					<FormControl>
						<JsonCode
							onChange={field.onChange}
							value={field.value}
						/>
					</FormControl>
					<FormDescription>
						Optional JSON object that will be passed to the Actor as
						input.
					</FormDescription>
					<FormMessage />
				</FormItem>
			)}
		/>
	);
};

// export const Tags = () => {
// 	// const setValues = useSetAtom(actorCustomTagValues);
// 	// const setKeys = useSetAtom(actorCustomTagKeys);

// 	const { data: tags = [] } = useInfiniteQuery(
// 		useManagerQueries().actorsTagsQueryOptions(),
// 	);

// 	const keys = useMemo(() => {
// 		return Array.from(
// 			new Set(tags.flatMap((tag) => Object.keys(tag))),
// 		).sort();
// 	}, [tags]);
// 	const values = useMemo(() => {
// 		return Array.from(
// 			new Set(tags.flatMap((tag) => Object.values(tag))),
// 		).sort();
// 	}, [tags]);

// 	return (
// 		<div className="space-y-2">
// 			<Label>Tags</Label>
// 			<TagsInput
// 				keys={keys.map((key) => ({
// 					label: key,
// 					value: key,
// 				}))}
// 				values={values.map((value) => ({
// 					label: value,
// 					value: value,
// 				}))}
// 				onCreateKeyOption={(value) => {
// 					// setKeys((old) =>
// 					// 	Array.from(new Set([...old, value]).values()),
// 					// );
// 				}}
// 				onCreateValueOption={(value) => {
// 					// setValues((old) =>
// 					// 	Array.from(new Set([...old, value]).values()),
// 					// );
// 				}}
// 			/>
// 		</div>
// 	);
// };
