import { writable, type Writable } from 'svelte/store';

export interface HeaderState {
	title: string;
	showBackButton: boolean;
	backPath?: string;
}

const initialHeaderState: HeaderState = {
	title: 'Scanner',
	showBackButton: false,
	backPath: undefined
};

export const headerState: Writable<HeaderState> = writable(initialHeaderState);

export const loadingState: Writable<boolean> = writable(false);
export function setLoading(loading: boolean) {
	loadingState.set(loading);
}

export function setHeader(newState: Partial<HeaderState>) {
	headerState.update((current) => ({ ...current, ...newState }));
}

export function resetHeader() {
	headerState.set(initialHeaderState);
}
