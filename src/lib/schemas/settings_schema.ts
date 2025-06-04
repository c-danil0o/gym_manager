import { z } from 'zod';

export const settingsSchema = z.object({
	language: z.string(),
	timezone: z.string(),
	theme: z.string(),
	backup_url: z.string().optional().nullable(),
	backup_period_hours: z.number().optional(),
});

export type SettingsSchemaType = typeof settingsSchema;
