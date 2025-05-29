import { z } from 'zod';

export const membershipSchema = z.object({
    member_id: z.number(),

    membership_id: z.number().optional().nullable(),
    membership_type_id: z.number().optional().nullable(),
    membership_start_date: z.string().optional().nullable(),
    membership_end_date: z.string().optional().nullable(),
    membership_remaining_visits: z.number().optional().nullable()
});

export type MembershipSchemaType = typeof membershipSchema;
