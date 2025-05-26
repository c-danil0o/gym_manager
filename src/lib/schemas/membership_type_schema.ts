import { z } from 'zod';

export const membershipTypeSchema = z.object({
    name: z.string().min(1, "Name is required"),
    duration_days: z.coerce.number().int().min(1).max(30).nullable().default(null),
    visit_limit: z.coerce.number().int().min(1).max(30).nullable().default(null),
    price: z.coerce.number().positive("Price must be positive"),
    description: z.string().optional().nullable().or(z.literal('')).default(''),
});

export type MembershipTypeSchema = typeof membershipTypeSchema;
