import { goto } from '$app/navigation';
import { auth } from '$lib/stores/auth.js';
import { get } from 'svelte/store';

export type UserRole = 'admin' | 'user';

export function requireRole(requiredRole: UserRole) {
	const authState = get(auth);
	if (authState.role !== requiredRole) {
		goto('/unauthorized');
	}
}
export function enabledForRole(requiredRole: UserRole): boolean {
	const authState = get(auth);
	return authState.role !== requiredRole;
}
