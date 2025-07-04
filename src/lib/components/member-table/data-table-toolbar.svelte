<script lang="ts" generics="TData">
	import XIcon from '@lucide/svelte/icons/x';
	import type { Table } from '@tanstack/table-core';
	import { DataTableFacetedFilter, DataTableViewOptions } from '../data-table/index.js';
	import Button from '$lib/components/ui/button/button.svelte';
	import { Input } from '$lib/components/ui/input/index.js';
	import { statuses } from './data.js';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { MembershipType } from '$lib/models/membership_type.js';
	import { PlusCircle } from 'lucide-svelte';
	import { m } from '$lib/paraglide/messages.js';

	let {
		table,
		onSearchChange,
		onAddMember
	}: { table: Table<TData>; onSearchChange?: (value: string) => void; onAddMember?: () => void } =
		$props();

	const isFiltered = $derived(table.getState().columnFilters.length > 0);
	const statusCol = $derived(table.getColumn('membership_status'));
	const membershipCol = $derived(table.getColumn('membership_type_name'));
	let membershipTypes = $state<{ label: string; value: string }[]>([]);

	async function fetchMembershipTypes() {
		try {
			const result = await invoke<MembershipType[]>('get_all_membership_types');
			if (result) {
				const types = result.map((type) => ({
					label: type.name,
					value: type.id.toString()
				}));
				membershipTypes = types;
			}
		} catch (e: any) {
			console.error('Error fetching membership types:', e);
		}
	}
	onMount(() => {
		fetchMembershipTypes();
	});
</script>

<div class="flex items-center justify-between">
	<div class="flex flex-1 items-center space-x-2">
		<Input
			placeholder={m.search()}
			oninput={(e) => {
				if (onSearchChange) {
					onSearchChange(e.currentTarget.value);
				}
			}}
			class="h-8 w-[150px] lg:w-[250px] bg-card"
		/>

		{#if statusCol}
			<DataTableFacetedFilter column={statusCol} title={m.status()} options={statuses} />
		{/if}

		{#if membershipCol}
			<DataTableFacetedFilter column={membershipCol} title={m.membership()} options={membershipTypes} />
		{/if}

		{#if isFiltered}
			<Button variant="ghost" onclick={() => table.resetColumnFilters()} class="h-8 px-2 lg:px-3">
				{m.reset()}
				<XIcon />
			</Button>
		{/if}
	</div>
	<Button onclick={onAddMember} class="h-8 px-4 ml-3 text-xs lg:px-4">
		<PlusCircle class="mr-2 h-4 w-4" />
		{m.add()}</Button
	>
</div>
