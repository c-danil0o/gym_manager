import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { cubicOut } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

type FlyAndScaleParams = {
	y?: number;
	x?: number;
	start?: number;
	duration?: number;
};

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
