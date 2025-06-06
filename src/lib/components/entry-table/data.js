import { m } from "$lib/paraglide/messages";

export const statuses = [
	{
		value: "allowed",
		label: m.allowed(),
	},
	{
		value: "allowed_single",
		label: m.allowed_single(),
	},
	{
		value: "denied_member_not_found",
		label: m.denied_member_not_found(),
	},
	{
		value: "denied_no_membership",
		label: m.denied_no_membership(),
	},
	{
		value: "denied_no_visits_left",
		label: m.denied_no_visits_left(),
	},
	{
		value: "denied_membership_expired",
		label: m.denied_membership_expired(),
	},

	{
		value: "denied_membership_not_active_yet",
		label: m.denied_membership_not_active_yet(),
	},
	{
		value: "denied_membership_inactive",
		label: m.denied_membership_inactive(),
	},
	{
		value: "denied_membership_suspended",
		label: m.denied_membership_suspended(),
	},
	{
		value: "denied_membership_invalid_status",
		label: m.denied_membership_invalid_status(),
	},
	{
		value: "denied_already_checked_in",
		label: m.denied_already_checked_in(),
	},
	{
		value: "denied_after_hours",
		label: m.denied_after_hours(),
	},
	{
		value: "error",
		label: m.error(),
	},
];
