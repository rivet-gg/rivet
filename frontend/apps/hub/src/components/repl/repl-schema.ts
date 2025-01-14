import { z } from "zod";

export const MessageSchema = z.object({
	type: z.literal("code"),
	data: z.string(),
	managerUrl: z.string(),
	actorId: z.string(),
	rpcs: z.array(z.string()),
	id: z.string(),
});

export const FormattedCodeSchema = z
	.object({
		fg: z.string(),
		tokens: z.array(
			z.array(
				z.object({
					content: z.string(),
					color: z.string(),
				}),
			),
		),
	})
	.catch((ctx) => ctx.input);

export const LogSchema = z.object({
	level: z.string(),
	message: z.any(),
});

export const ResponseSchema = z.discriminatedUnion("type", [
	z.object({
		type: z.literal("error"),
		id: z.string(),
		data: z.any(),
	}),
	z.object({
		type: z.literal("formatted"),
		id: z.string(),
		data: FormattedCodeSchema,
	}),
	z.object({
		type: z.literal("result"),
		id: z.string(),
		data: z.any(),
	}),
	z.object({
		type: z.literal("log"),
		id: z.string(),
		data: LogSchema,
	}),
]);

export type Response = z.infer<typeof ResponseSchema>;
export type Message = z.infer<typeof MessageSchema>;
export type FormattedCode = z.infer<typeof FormattedCodeSchema>;
export type Log = z.infer<typeof LogSchema>;
