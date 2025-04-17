import { convertStringToId } from "@/lib/utils";
import {
	Button,
	Combobox,
	createSchemaForm,
	FormControl,
	FormField,
	FormFieldContext,
	FormItem,
	FormLabel,
	FormMessage,
	Text,
	Input,
	type ComboboxOption,
	FormDescription,
} from "@rivet-gg/components";
import { Icon, faTrash } from "@rivet-gg/icons";
import {
	type UseFormReturn,
	useFieldArray,
	useFormContext,
} from "react-hook-form";
import z from "zod";

export const formSchema = z.object({
	routeName: z
		.string()
		.max(25)
		.refine((value) => value.trim() !== "" && value.trim() === value, {
			message: "Name cannot be empty or contain whitespaces",
		}),
	slug: z.string().max(25).optional(),
	hostname: z
		.string()
		.min(1)
		.refine((value) => {
			const regex = /^[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)*$/;
			return regex.test(value);
		}, "Hostname must be a valid domain name"),
	path: z
		.string()
		.min(1)
		.refine((value) => {
			const regex = /^\/[a-zA-Z0-9-_\/]*\*?$/;
			return regex.test(value);
		}, "Path must start with a / and contain only alphanumeric characters, dashes, and underscores")
		.refine((value) => {
			const endsWithSlash = value.endsWith("/");
			return !endsWithSlash;
		}, "Path must not end with a /"),
	tags: z
		.array(
			z.object({
				key: z.string().min(1),
				value: z.string(),
			}),
		)
		.superRefine((tags, ctx) => {
			const tagsSet = new Set<string>();
			for (const [idx, tag] of [...tags].reverse().entries()) {
				if (tagsSet.has(tag.key)) {
					ctx.addIssue({
						code: z.ZodIssueCode.custom,
						path: [idx, "key"],
						message: "Tag keys must be unique",
					});
				}
				tagsSet.add(tag.key);
			}
		}),
});

export type FormValues = z.infer<typeof formSchema>;
export type SubmitHandler = (
	values: FormValues,
	form: UseFormReturn<FormValues>,
) => Promise<void>;

const { Form, Submit } = createSchemaForm(formSchema);
export { Form, Submit };

export const Name = ({ className }: { className?: string }) => {
	const { control } = useFormContext<FormValues>();
	return (
		<FormField
			control={control}
			name="routeName"
			render={({ field }) => (
				<FormItem className={className}>
					<FormLabel className="col-span-1">Route Name</FormLabel>
					<FormControl className="row-start-2">
						<Input
							placeholder="Enter a route name..."
							maxLength={25}
							autoComplete="off"
							{...field}
						/>
					</FormControl>
					<FormMessage className="col-span-1" />
				</FormItem>
			)}
		/>
	);
};

export const Slug = ({ className }: { className?: string }) => {
	const { control, watch } = useFormContext<FormValues>();

	const name = watch("routeName");

	return (
		<FormField
			control={control}
			name="slug"
			render={({ field }) => (
				<FormItem className={className}>
					<FormLabel className="col-span-2">Slug</FormLabel>
					<FormControl className="row-start-2">
						<Input
							placeholder={
								name
									? convertStringToId(name)
									: "Enter a slug..."
							}
							maxLength={25}
							{...field}
							onChange={(event) => {
								const value = convertStringToId(
									event.target.value,
								);
								field.onChange({ target: { value } });
							}}
						/>
					</FormControl>
					<FormMessage className="col-span-2" />
				</FormItem>
			)}
		/>
	);
};

export const Hostname = () => {
	const { control } = useFormContext<FormValues>();
	return (
		<FormField
			control={control}
			name="hostname"
			render={({ field }) => (
				<FormItem className="flex-1">
					<FormLabel>Hostname</FormLabel>
					<FormControl>
						<Input
							type="text"
							className="input"
							placeholder="example.com"
							{...field}
						/>
					</FormControl>
					<FormMessage />
				</FormItem>
			)}
		/>
	);
};

export const Path = () => {
	const { control, watch } = useFormContext<FormValues>();

	return (
		<FormField
			control={control}
			name="path"
			render={({ field, fieldState }) => (
				<FormItem className="flex-1">
					<FormLabel>Path</FormLabel>
					<FormControl>
						<Input
							type="text"
							className="input"
							placeholder="/api/v1"
							{...field}
						/>
					</FormControl>
					<FormMessage />
					<FormDescription>
						<PathMessage />
					</FormDescription>
				</FormItem>
			)}
		/>
	);
};

const PathMessage = () => {
	const { watch } = useFormContext<FormValues>();

	const path = watch("path");

	return path.endsWith("/*") ? (
		<span>Maximum of 8 path components when routing path prefixes.</span>
	) : null;
};

export const Tags = ({
	onCreateKeyOption,
	onCreateValueOption,
	keys,
	values,
}: {
	onCreateKeyOption: (option: string) => void;
	onCreateValueOption: (option: string) => void;
	keys: ComboboxOption[];
	values: ComboboxOption[];
}) => {
	const { control, setValue, watch } = useFormContext<FormValues>();
	const { fields, append, remove } = useFieldArray({
		name: "tags",
		control,
	});

	return (
		<>
			{fields.length === 0 ? (
				<Text mb="2" className="text-xs">
					There's no selectors added.
				</Text>
			) : null}
			{fields.map((field, index) => (
				<div
					key={field.id}
					className="grid grid-cols-[1fr,1fr,auto] grid-rows-[repeat(3,auto)] items-start mt-2 gap-2"
				>
					<FormFieldContext.Provider
						value={{ name: `tags.${index}.key` }}
					>
						<FormItem
							flex="1"
							className="grid grid-cols-subgrid grid-rows-subgrid row-span-full"
						>
							<FormLabel>Key</FormLabel>
							<FormControl>
								<Combobox
									placeholder="Choose a key"
									options={keys}
									className="w-full"
									value={watch(`tags.${index}.key`)}
									onValueChange={(value) => {
										setValue(`tags.${index}.key`, value, {
											shouldDirty: true,
											shouldTouch: true,
											shouldValidate: true,
										});
									}}
									allowCreate
									onCreateOption={onCreateKeyOption}
								/>
							</FormControl>
							<FormMessage />
						</FormItem>
					</FormFieldContext.Provider>

					<FormFieldContext.Provider
						value={{ name: `tags.${index}.value` }}
					>
						<FormItem
							flex="1"
							className="grid grid-cols-subgrid grid-rows-subgrid row-span-full"
						>
							<FormLabel>Value</FormLabel>
							<FormControl>
								<Combobox
									placeholder="Choose a value"
									options={values}
									className="w-full"
									value={watch(`tags.${index}.value`)}
									onValueChange={(value) => {
										setValue(`tags.${index}.value`, value, {
											shouldDirty: true,
											shouldTouch: true,
											shouldValidate: true,
										});
									}}
									allowCreate
									onCreateOption={onCreateValueOption}
								/>
							</FormControl>
							<FormMessage />
						</FormItem>
					</FormFieldContext.Provider>
					<Button
						size="icon"
						className="self-end row-start-2"
						variant="secondary"
						type="button"
						onClick={() => remove(index)}
					>
						<Icon icon={faTrash} />
					</Button>
				</div>
			))}
			<Button
				className="justify-self-start"
				variant="secondary"
				size="sm"
				type="button"
				onClick={() => append({ value: "", key: "" })}
			>
				Add a selector
			</Button>
		</>
	);
};
