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
		backup_period_hours: z.number().optional(),
		sync_enabled: z.boolean(),
		supabase_url: z.string().optional().nullable(),
		supabase_key: z.string().optional().nullable(),
		supabase_jwt_secret: z.string().optional().nullable(),
		sync_period_seconds: z.number().optional()
	})
	.refine(
		(data) => {
			if (data.backup_enabled && (!data.backup_url || data.backup_url === '')) {
				return false;
			}
			return true;
		},
		{ message: m.backup_url_not_set(), path: ['backup_url'] }
	)
	.refine(
		(data) => {
			if (data.sync_enabled && (!data.supabase_jwt_secret || data.supabase_jwt_secret === '')) {
				return false;
			}
			return true;
		},
		{ message: m.supabase_jwt_required(), path: ['supabase_jwt_secret'] }
	)
	.refine(
		(data) => {
			if (data.sync_enabled && (!data.supabase_url || data.supabase_url === '')) {
				return false;
			}
			return true;
		},
		{ message: m.supabase_url_required(), path: ['supabase_url'] }
	)
	.refine(
		(data) => {
			if (data.sync_enabled && (!data.supabase_key || data.supabase_key === '')) {
				return false;
			}
			return true;
		},
		{ message: m.supabase_key_required(), path: ['supabase_key'] }
	);

export type SettingsSchemaType = typeof settingsSchema;
