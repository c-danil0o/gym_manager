import { z } from 'zod';

export const memberWithMembershipSchema = z.object({
    id: z.number(),
    first_name: z.string().min(1, "First name is required"),
    last_name: z.string().min(1, "Last name is required"),
    card_id: z.string().min(6, "Card number must be exactly 6 characters").max(6, "Card number must be exactly 6 characters"),
    email: z.string().email("Invalid email format").optional().nullable(),
    phone: z.string().optional().nullable(),
    date_of_birth: z.string().optional().nullable(),

    membership_id: z.number().optional().nullable(),
    membership_type_id: z.number().optional().nullable(),
    start_date: z.string().optional().nullable(),
    end_date: z.string().optional().nullable(),
    remaining_visits: z.number().optional().nullable()
});

export type MemberWithMembershipSchema = typeof memberWithMembershipSchema;
;
