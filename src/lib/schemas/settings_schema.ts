import { m } from '$lib/paraglide/messages';
import { z } from 'zod';

export const settingsSchema = z
	.object({
		language: z.string(),
		timezone: z.string(),
		theme: z.string(),
		gym_name: z.string(),
		backup_enabled: z.boolean(),
		backup_url: z.string().optional().nullable(),
		backup_period_hours: z.number().optional()
	})
	.refine(
		(data) => {
			if (data.backup_enabled && (!data.backup_url || data.backup_url === '')) {
				return false;
			}
			return true;
		},
		{ message: m.backup_url_not_set(), path: ['backup_url'] }
	);

export type SettingsSchemaType = typeof settingsSchema;
