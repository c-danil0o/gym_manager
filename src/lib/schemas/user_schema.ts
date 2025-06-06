import { m } from '$lib/paraglide/messages';
import { z } from 'zod';

export const newUserSchema = z.object({
	username: z.string().min(1, m.username_required()),
	role: z.string().min(1, m.role_required()),
	password: z.string().min(6, m.password_min_length())
});

export type NewUserSchemaType = typeof newUserSchema;

export const updateUserSchema = z.object({
	id: z.number().optional(),
	username: z.string().min(1, m.username_required()),
	role: z.string().min(1, m.role_required())
});

export type UpdateUserSchemaType = typeof updateUserSchema;

export const changePasswordSchema = z.object({
	user_id: z.number(),
	new_password: z.string().min(6, m.password_min_length())
});

export type ChangePasswordSchemaType = typeof changePasswordSchema;
