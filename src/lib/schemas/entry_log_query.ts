import { z } from 'zod';

export const entryLogQuerySchema = z.object({
	member_name: z.string().optional(),
	card_id: z.string().optional(),
	status: z.string().optional(),
	date_from: z.string().optional(),
	date_to: z.string().optional(),
	page: z.number().optional(),
	per_page: z.number().optional(),
	order_by: z.string().optional(),
	order_by_direction: z.string().optional(),
});

export type EntryLogQuerySchemaType = typeof entryLogQuerySchema;
