import { m } from "$lib/paraglide/messages";

export const statuses = [
	{
		value: "inactive",
		label: m.inactive(),
	},
	{
		value: "expired",
		label: m.expired(),
	},
	{
		value: "active",
		label: m.active(),
	},
	{
		value: "pending",
		label: m.pending(),
	},
	{
		value: "suspended",
		label: m.suspended(),
	},
];
