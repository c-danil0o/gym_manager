<script lang="ts">
	import { goto } from '$app/navigation';
	import { EntryLogDataTable } from '$lib/components/entry-table';
	import type { EntryLog } from '$lib/models/entry';
	import type { FilterField, QueryRequest, QueryResponse } from '$lib/models/table-state';
	import { m } from '$lib/paraglide/messages';
	import { setHeader, setLoading } from '$lib/stores/state';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import Label from '$lib/components/ui/label/label.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	onMount(() => {
		setLoading(true);
		setHeader({
			title: 'Entry Log',
			showBackButton: false
		});
	});
	let tableData = $state<QueryResponse<EntryLog>>({
		data: [],
		total: 0,
		page: 1,
		per_page: 10,
		total_pages: 0
	});

	let todayDate: DateValue = today(getLocalTimeZone());
	let startDate: DateValue = today(getLocalTimeZone()).subtract({ months: 6 });
	let deleteDialogOpen = $state(false);
	let selectedPeriod = $state<string | undefined>(null);

	let loading = $state(false);
	let currentParams = $state<QueryRequest>({
		page: 1,
		per_page: 10,
		order_by: undefined,
		order_direction: undefined,
		search_string: '',
		filter_fields: [],
		date_from: startDate ? startDate.toString() : undefined,
		date_to: todayDate ? todayDate.toString() : undefined
	});

	// Reference to the table component for accessing methods
	let tableRef: any;

	// Fetch data from backend
	async function fetchTableData(params: QueryRequest) {
		loading = true;
		try {
			const response = await invoke<QueryResponse<EntryLog>>('get_entry_logs', {
				searchParams: {
					page: params.page,
					per_page: params.per_page,
					order_by: params.order_by,
					order_direction: params.order_direction,
					search_string: params.search_string || '',
					filter_fields: params.filter_fields || [],
					date_from: params.date_from,
					date_to: params.date_to
				}
			});

			tableData = response;
		} catch (error) {
			console.error('Failed to fetch entry log data:', error);
			toast.error(m.failed_to_fetch_entry_log());
		} finally {
			loading = false;
			setLoading(false);
		}
	}
	function handleViewMember(id: number | null) {
		if (!id) return;
		goto(`/members/${id}`);
	}

	// Event handlers
	function handlePageChange(page: number) {
		currentParams = { ...currentParams, page };
	}

	function handlePageSizeChange(pageSize: number) {
		currentParams = { ...currentParams, per_page: pageSize, page: 1 };
	}

	function handleSortChange(orderBy: string | null, orderDirection: 'asc' | 'desc' | null) {
		currentParams = {
			...currentParams,
			order_by: orderBy || undefined,
			order_direction: orderDirection || undefined,
			page: 1 // Reset to first page when sorting changes
		};
	}

	function handleClearLog() {
		deleteDialogOpen = true;
	}

	function handleSearchChange(searchString: string) {
		currentParams = {
			...currentParams,
			search_string: searchString,
			page: 1 // Reset to first page when search changes
		};
	}
	function handleStartDateChange(startDate: DateValue | undefined) {
		if (startDate) {
			currentParams = {
				...currentParams,
				date_from: startDate.toString(),
				page: 1
			};
		} else {
			currentParams = { ...currentParams, date_from: undefined };
		}
	}

	function handleEndDateChange(endDate: DateValue | undefined) {
		console.log(endDate);
		if (endDate) {
			currentParams = {
				...currentParams,
				date_to: endDate.toString(),
				page: 1
			};
		} else {
			currentParams = { ...currentParams, date_to: undefined };
		}
	}

	function handleFilterChange(filterFields: FilterField[]) {
		currentParams = {
			...currentParams,
			filter_fields: filterFields,
			page: 1 // Reset to first page when filters change
		};
	}

	let searchTimeout: any;
	function debouncedSearchChange(searchString: string) {
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			handleSearchChange(searchString);
		}, 300);
	}
	async function handleDelete(id: number | null) {
		if (!id) return;

		try {
			await invoke('delete_entry_log', { entryLogId: id });
			fetchTableData(currentParams); // Refresh data after deletion
			toast.success(m.entry_log_delete_success());
		} catch (e: any) {
			console.error('Error deleting entry log:', e);
			toast.error(m.entry_log_delete_fail());
		}
	}

	async function handleDeleteLogs() {
		if (!selectedPeriod) {
			toast.error(m.select_period());
			return;
		}

		try {
			setLoading(true);
			await invoke('delete_entry_logs', { period: Number.parseInt(selectedPeriod) });
			deleteDialogOpen = false;
			selectedPeriod = undefined;
			fetchTableData(currentParams);
			toast.success(m.entry_log_delete_success());
		} catch (e: any) {
			console.error('Error deleting entry logs:', e);
			toast.error(m.entry_log_delete_fail());
		}
	}

	// Load initial data
	$effect(() => {
		fetchTableData(currentParams);
	});
</script>

<div>
	<EntryLogDataTable
		bind:this={tableRef}
		serverData={tableData}
		{loading}
		onPageChange={handlePageChange}
		onPageSizeChange={handlePageSizeChange}
		onSortChange={handleSortChange}
		onSearchChange={debouncedSearchChange}
		onFilterChange={handleFilterChange}
		onStartDateChange={handleStartDateChange}
		onEndDateChange={handleEndDateChange}
		{handleDelete}
		onRowClick={handleViewMember}
		{handleClearLog}
	/>
	<Dialog.Root open={deleteDialogOpen}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>{m.entry_log_delete()}</Dialog.Title>
				<Dialog.Description>
					{m.entry_log_delete_desc()}
				</Dialog.Description>
			</Dialog.Header>
			<div class="flex items-center justify-center w-full">
				<Label class="mr-4">{m.save_last()}:</Label>
				<Select.Root type="single" bind:value={selectedPeriod}>
					<Select.Trigger class="w-fit">
						{selectedPeriod
							? `${selectedPeriod} ${Number(selectedPeriod) > 1 ? m.months() : m.month()}`
							: m.select_period()}
					</Select.Trigger>
					<Select.Content>
						<Select.Group>
							<Select.Item value="1" label={'1 ' + m.months()}></Select.Item>
							<Select.Item value="3" label={'3 ' + m.monthsa()}></Select.Item>
							<Select.Item value="6" label={'6 ' + m.months()}></Select.Item>
							<Select.Item value="12" label={'12 ' + m.months()}></Select.Item>
							<Select.Item value="24" label={'24 ' + m.months()}></Select.Item>
							<Select.Item value="36" label={'36 ' + m.months()}></Select.Item>
						</Select.Group>
					</Select.Content>
				</Select.Root>
			</div>

			<div class="flex gap-20 justify-around">
				<Button variant="outline" onclick={() => (deleteDialogOpen = false)} class="w-full"
					>{m.cancel()}</Button
				>
				<Button onclick={handleDeleteLogs} class="w-full">{m.confirm()}</Button>
			</div>
		</Dialog.Content>
	</Dialog.Root>
</div>
