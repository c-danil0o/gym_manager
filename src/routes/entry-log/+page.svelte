<script lang="ts">
	import { EntryLogDataTable } from '$lib/components/entry-table';
	import type { EntryLog } from '$lib/models/entry';
	import type { FilterField, QueryRequest, QueryResponse } from '$lib/models/table-state';
	import { setHeader, setLoading } from '$lib/stores/state';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
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
			// Handle error appropriately
		} finally {
			loading = false;
			setLoading(false);
		}
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
			toast.success('Entry log deleted successfully.');
		} catch (e: any) {
			console.error('Error deleting entry log:', e);
			toast.error(e?.message || 'Failed to entry log.');
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
	/>
</div>
