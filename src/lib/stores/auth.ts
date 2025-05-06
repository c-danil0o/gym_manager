import { writable } from 'svelte/store';

interface LoginResponse {
	username: string;
	message: string;
	success: boolean;
}

function createAuthStore() {
	const { subscribe, set, update } = writable({
		isAuthenticated: false,
		username: null as string | null,
		error: null as string | null
	});

	return {
		subscribe,
		login: async (username: string, password: string) => {
			const { invoke } = await import('@tauri-apps/api/core');
			try {
				const response: LoginResponse = await invoke('login_admin', {
					payload: { username, password }
				});

				if (response.success) {
					set({ isAuthenticated: true, username: response.username, error: null });
				} else {
					set({ isAuthenticated: false, username: null, error: response.message });
					return false;
				}
				return true;
			} catch (e) {
				console.error('Login error:', e);
				set({
					isAuthenticated: false,
					username: null,
					error: e.message || 'An unexpected error occurred.'
				});
				return false;
			}
		},
		logout: () => {
			set({ isAuthenticated: false, username: null, error: null });
		},
		clearError: () => {
			update((state) => ({ ...state, error: null }));
		}
	};
}

export const auth = createAuthStore();
