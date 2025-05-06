import { z } from 'zod';

export const formSchema = z.object({
	username: z
		.string()
		.min(2, { message: 'Username must be between 2 and 50 characters' })
		.max(20, { message: 'Username must be between 2 and 50 characters' }),
	password: z
		.string()
		.min(5, { message: 'Password must be between 5 and 20 characters' })
		.max(20, { message: 'Password must be between 5 and 20 characters' })
});

export type LoginSchema = typeof formSchema;
