import { writable, type Writable } from 'svelte/store';

export interface HeaderState {
    title: string;
    showBackButton: boolean;
    backPath?: string;
}

const initialHeaderState: HeaderState = {
    title: 'Dashboard',
    showBackButton: false,
    backPath: undefined,
};

export const headerState: Writable<HeaderState> = writable(initialHeaderState);

export function setHeader(newState: Partial<HeaderState>) {
    headerState.update(current => ({ ...current, ...newState }));
}

export function resetHeader() {
    headerState.set(initialHeaderState);
}
