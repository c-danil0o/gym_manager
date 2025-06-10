import { m } from '$lib/paraglide/messages';
import { z } from 'zod';

export const membershipTypeSchema = z
	.object({
		name: z.string().min(1, m.name_required()),
		duration_days: z.coerce
			.number({ message: m.invalid_character() })
			.int()
			.min(1, m.duration_min())
			.max(366, m.duration_max())
			.nullable()
			.default(null),
		visit_limit: z.coerce
			.number({ message: m.invalid_character() })
			.int()
			.min(0, m.visit_limit_min())
			.max(366, m.visit_limit_max())
			.nullable()
			.default(null),
		enter_by: z.coerce
			.number({ message: m.invalid_character() })
			.int()
			.min(1, m.entry_by_min_max())
			.max(24, m.entry_by_min_max())
			.nullable()
			.default(null),
		price: z.coerce.number({ message: m.invalid_character() }).positive(m.price_positive()),
		description: z.string().optional().nullable().or(z.literal('')).default(''),
		is_active: z.boolean().default(true),
	})
	.refine(
			(data) => {
				if (data.visit_limit !== null && data.duration_days !== null) {
					return data.visit_limit <= data.duration_days;
				}
				return true;
			},
			{
				message: m.visit_limit_greater_than_duration(),
				path: ['visit_limit']
			}
		);
export type MembershipTypeSchema = typeof membershipTypeSchema;
