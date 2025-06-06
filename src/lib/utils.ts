import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { cubicOut } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';
import { m } from './paraglide/messages';
import { goto } from '$app/navigation';
import { browser } from '$app/environment';
import { auth } from './stores/auth';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

type FlyAndScaleParams = {
	y?: number;
	x?: number;
	start?: number;
	duration?: number;
};

export function translateEntryMessage(message: string): string {
	const [key, param] = message.split('|');

	switch (key) {
		case 'card_invalid':
			return m['scan_message.card_invalid']();
		case 'member_not_found':
			return m['scan_message.member_not_found']();
		case 'no_membership':
			return m['scan_message.no_membership']();
		case 'invalid_membership':
			return m['scan_message.invalid_membership']();
		case 'no_visits_left':
			return m['scan_message.no_visits_left']();
		case 'expired_on':
			return m['scan_message.expired_on']({ date: param ?? '???' });
		case 'pending':
			return m['scan_message.pending']({ date: param ?? '???' });
		case 'inactive':
			return m['scan_message.inactive']();
		case 'suspended':
			return m['scan_message.suspended']();
		case 'invalid_status':
			return m['scan_message.invalid_status']();
		case 'after_hours':
			return m['scan_message.after_hours']({ hour: param ?? '???' });
		case 'already_checked':
			return m['scan_message.already_checked']();
		case 'error':
			return m['scan_message.error']();
		case 'allowed':
			return m['scan_message.allowed']();
		case 'allowed_single':
			return m['scan_message.allowed_single']();
		default:
			return message; // fallback if key isn't known
	}
}
export function translateAccessStatus(status: string | null): string {
	if (!status) return '';

	switch (status) {
		case 'allowed':
			return m.allowed();
		case 'allowed_single':
			return m.allowed_single();
		case 'denied_member_not_found':
			return m.denied_member_not_found();
		case 'denied_no_membership':
			return m.denied_no_membership();
		case 'denied_no_visits_left':
			return m.denied_no_visits_left();
		case 'denied_membership_expired':
			return m.denied_membership_expired();
		case 'denied_membership_not_active_yet':
			return m.denied_membership_not_active_yet();
		case 'denied_membership_inactive':
			return m.denied_membership_inactive();
		case 'denied_membership_suspended':
			return m.denied_membership_suspended();
		case 'denied_membership_invalid_status':
			return m.denied_membership_invalid_status();
		case 'denied_already_checked_in':
			return m.denied_already_checked_in();
		case 'denied_after_hours':
			return m.denied_after_hours();
		case 'error_updating_membership':
			return m.error_updating_membership();
		case 'error':
			return m.error();
		default:
			return status; // Return original status if no translation found
	}
}

export function translateStatus(status: string | null): string {
	switch (status?.toLowerCase()) {
		case 'active':
			return m.active();
		case 'expired':
			return m.expired();
		case 'pending':
			return m.pending();
		case 'suspended':
			return m.suspended();
		default:
			return m.inactive();
	}
}

export function translateRole(role: string | null): string {
	switch (role?.toLowerCase()) {
		case 'user':
			return m.user();
		case 'admin':
			return m.admin();
		default:
			return '-';
	}
}
export function translateErrorCode(errorCode: string, params: any) {
	switch (errorCode) {
		case 'error.card_already_exists':
			return params?.card_id
				? m.error_card_already_exists({ cardId: params.card_id })
				: m.error_card_already_exists_generic();

		case 'error.membership_type_name_exists':
			return params?.name
				? m.error_membership_name_already_exists({ name: params.name })
				: m.error_membership_name_already_exists_generic();

		case 'error.overlapping_membership':
			return params?.id
				? m.error_overlapping_membership({ id: params?.id })
				: m.error_overlapping_membership({ id: '' });

		case 'error.username_already_exists':
			return m.username_already_exists();

		default:
			return m.error_unknown_error();
	}
}

export function getStatusClasses(status: string): string {
	switch (status?.toLowerCase()) {
		case 'active':
			return cn('border-green-500 bg-green-50 text-green-800 focus-visible:ring-green-500');
		case 'expired':
			return cn('border-red-500 bg-red-50 text-red-800 focus-visible:ring-red-500');
		case 'pending':
			return cn('border-yellow-500 bg-yellow-50 text-yellow-800 focus-visible:ring-yellow-500');
		case 'suspended':
			return cn('border-orange-500 bg-orange-50 text-orange-800 focus-visible:ring-orange-500');
		default:
			return cn('border-input bg-background');
	}
}

export function getSubtleStatusClasses(status: string): string {
	switch (status?.toLowerCase()) {
		case 'active':
			return cn('border-l-8 border-l-green-500! bg-card');
		case 'expired':
			return cn('border-l-8 border-l-red-500! bg-card');
		case 'pending':
			return cn('border-l-8 border-l-yellow-500! bg-card');
		case 'suspended':
			return cn('border-l-8 border-l-orange-500! bg-card');
		case 'inactive':
			return cn('border-l-8 border-l-blue-500! bg-card');
		default:
			return cn('border-input bg-card');
	}
}

export function getMembershipStatusBadgeVariant(
	status: string | null
): 'default' | 'secondary' | 'destructive' | 'outline' {
	switch (status?.toLowerCase()) {
		case 'active':
			return 'default'; // Or a success color if you have one
		case 'expired':
			return 'destructive';
		case 'pending':
			return 'secondary';
		case 'cancelled':
			return 'outline';
		default:
			return 'secondary';
	}
}

export const flyAndScale = (
	node: Element,
	params: FlyAndScaleParams = { y: -8, x: 0, start: 0.95, duration: 150 }
): TransitionConfig => {
	const style = getComputedStyle(node);
	const transform = style.transform === 'none' ? '' : style.transform;

	const scaleConversion = (valueA: number, scaleA: [number, number], scaleB: [number, number]) => {
		const [minA, maxA] = scaleA;
		const [minB, maxB] = scaleB;

		const percentage = (valueA - minA) / (maxA - minA);
		const valueB = percentage * (maxB - minB) + minB;

		return valueB;
	};

	const styleToString = (style: Record<string, number | string | undefined>): string => {
		return Object.keys(style).reduce((str, key) => {
			if (style[key] === undefined) return str;
			return str + key + ':' + style[key] + ';';
		}, '');
	};

	return {
		duration: params.duration ?? 200,
		delay: 0,
		css: (t) => {
			const y = scaleConversion(t, [0, 1], [params.y ?? 5, 0]);
			const x = scaleConversion(t, [0, 1], [params.x ?? 0, 0]);
			const scale = scaleConversion(t, [0, 1], [params.start ?? 0.95, 1]);

			return styleToString({
				transform: transform + 'translate3d(' + x + 'px, ' + y + 'px, 0) scale(' + scale + ')',
				opacity: t
			});
		},
		easing: cubicOut
	};
};
