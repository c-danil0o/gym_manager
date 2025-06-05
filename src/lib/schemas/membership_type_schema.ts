import { m } from '$lib/paraglide/messages';
import { z } from 'zod';

export const membershipTypeSchema = z.object({
    name: z.string().min(1, m.name_required()),
    duration_days: z.coerce.number().int().min(1, m.duration_min()).max(366, m.duration_max()).nullable().default(null),
    visit_limit: z.coerce.number().int().min(0, m.visit_limit_min()).max(366, m.visit_limit_max()).nullable().default(null),
    enter_by: z.coerce.number().int().min(1, m.entry_by_min_max()).max(24, m.entry_by_min_max()).nullable().default(null),
    price: z.coerce.number().positive(m.price_positive()),
    description: z.string().optional().nullable().or(z.literal('')).default(''),
});

export type MembershipTypeSchema = typeof membershipTypeSchema;
