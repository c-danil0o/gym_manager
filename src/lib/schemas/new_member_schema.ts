import { z } from 'zod';

export const newMemberSchema = z.object({
    first_name: z.string().min(1, "First name is required"),
    last_name: z.string().min(1, "Last name is required"),
    card_id: z.string().min(6, "Card number must be exactly 6 characters").max(6, "Card number must be exactly 6 characters"),
    email: z.string().email("Invalid email format").or(z.literal("")).optional().nullable(),
    phone: z.string().optional().nullable(),
    date_of_birth: z.string().optional().nullable(),
});

export type NewMemberTypeSchema = typeof newMemberSchema;
