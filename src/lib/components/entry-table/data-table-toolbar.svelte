<script lang="ts" generics="TData">
	import XIcon from '@lucide/svelte/icons/x';
	import type { Table } from '@tanstack/table-core';
	import { DataTableFacetedFilter, DataTableViewOptions } from '../data-table/index.js';
	import Button from '$lib/components/ui/button/button.svelte';
	import { Input } from '$lib/components/ui/input/index.js';
	import { statuses } from './data.js';
	import Label from '../ui/label/label.svelte';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';
	import { m } from '$lib/paraglide/messages.js';
	import DatePicker from '../date-picker/date-picker.svelte';

	let {
		table,
		onSearchChange,
		onStartDateChange,
		onEndDateChange,
		handleClearLog
	}: {
		table: Table<TData>;
		onSearchChange?: (value: string) => void;
		onStartDateChange?: (value: DateValue | undefined) => void;
		onEndDateChange?: (value: DateValue | undefined) => void;
		handleClearLog?: () => void;

	} = $props();

	let todayDate: DateValue = today(getLocalTimeZone());
	let startDate: DateValue = today(getLocalTimeZone()).subtract({ months: 6 });

	const isFiltered = $derived(table.getState().columnFilters.length > 0);
	const statusCol = $derived(table.getColumn('status'));
</script>

<div class="flex items-center justify-between">
	<div class="flex flex-1 items-center space-x-6">
		<Input
			placeholder={m.search()}
			oninput={(e) => {
				if (onSearchChange) {
					onSearchChange(e.currentTarget.value);
				}
			}}
			class="h-8 w-[150px] lg:w-[250px] bg-card"
		/>
		<div class="flex items-center gap-6">
			<div class="flex items-center gap-3">
				<Label class="text-xs text-muted-foreground">{m.from()}</Label>
				<DatePicker height={'h-8 py-1'} onValueChange={onStartDateChange} value={startDate} locale={m.locale_code()} />
			</div>
			<div class="flex items-center gap-3">
				<Label class="text-xs text-muted-foreground">{m.to()}</Label>
				<DatePicker height={'h-8 py-1'} onValueChange={onEndDateChange} value={todayDate} maxValue={todayDate} locale={m.locale_code()} />
			</div>
		</div>

		{#if statusCol}
			<DataTableFacetedFilter column={statusCol} title={m.status()} options={statuses} />
		{/if}

		{#if isFiltered}
			<Button variant="ghost" onclick={() => table.resetColumnFilters()} class="h-8 px-2 lg:px-3">
				{m.reset()}
				<XIcon />
			</Button>
		{/if}
	</div>

	<Button variant="outline" onclick={handleClearLog} class="h-8 px-2 lg:px-3 mr-5 text-xs">
		{m.clear_entry_log()}
	</Button>
	<DataTableViewOptions {table} />
</div>
