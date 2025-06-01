<script lang="ts">
	import {
		type ColumnDef,
		type ColumnFiltersState,
		type PaginationState,
		type Row,
		type RowSelectionState,
		type SortingState,
		type VisibilityState,
		type Table as TableType,
		getCoreRowModel,
		type Column
	} from '@tanstack/table-core';
	import DataTableToolbar from './data-table-toolbar.svelte';
	import { createSvelteTable } from '$lib/components/ui/data-table/data-table.svelte.js';
	import FlexRender from '$lib/components/ui/data-table/flex-render.svelte';
	import * as Table from '$lib/components/ui/table/index.js';
	import { statuses } from './data.js';
	import { renderSnippet } from '$lib/components/ui/data-table/render-helpers.js';
	import { createRawSnippet } from 'svelte';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronsLeftIcon from '@lucide/svelte/icons/chevrons-left';
	import ChevronsRightIcon from '@lucide/svelte/icons/chevrons-right';
	import ArrowUpIcon from '@lucide/svelte/icons/arrow-up';
	import ArrowDownIcon from '@lucide/svelte/icons/arrow-down';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import * as Select from '$lib/components/ui/select/index.js';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils.js';
	import { Trash2 } from 'lucide-svelte';
	import type { EntryLog } from '$lib/models/entry';
	import { parseDateTime, type DateValue } from '@internationalized/date';

	// Server-side data structure
	interface TableData<T> {
		data: T[];
		total: number;
		page: number;
		per_page: number;
		total_pages: number;
	}

	// Filter field structure matching your backend
	interface FilterField {
		field: string;
		value: string;
	}

	// Props interface
	interface Props {
		serverData: TableData<EntryLog>;
		loading?: boolean;
		onPageChange: (page: number) => void;
		onPageSizeChange: (pageSize: number) => void;
		onSortChange: (orderBy: string | null, orderDirection: 'asc' | 'desc' | null) => void;
		onSearchChange: (searchString: string) => void;
		onFilterChange: (filterFields: FilterField[]) => void;
		handleDelete?: (logId: string) => void;
		onStartDateChange?: (value: DateValue | undefined) => void;
		onEndDateChange?: (value: DateValue | undefined) => void;
	}

	let {
		serverData = [{ data: [], total: 0, page: 1, per_page: 10, total_pages: 1 }],
		loading = false,
		onPageChange,
		onPageSizeChange,
		onSortChange,
		onSearchChange,
		onFilterChange,
		handleDelete = () => {},
		onStartDateChange = () => {},
		onEndDateChange = () => {}
	}: Props = $props();

	// Local state for UI only
	let rowSelection = $state<RowSelectionState>({});
	let columnVisibility = $state<VisibilityState>({});

	// Server-controlled state
	let columnFilters = $state<ColumnFiltersState>([]);
	let sorting = $state<SortingState>([]);
	let pagination = $state<PaginationState>({
		pageIndex: serverData.page - 1 || 0,
		pageSize: serverData.per_page ?? 10
	});

	// Update pagination when server data changes
	$effect(() => {
		pagination = {
			pageIndex: serverData.page - 1,
			pageSize: serverData.per_page
		};
	});

	// Handle server-side sorting
	function handleSort(columnId: string, desc: boolean) {
		const newSorting = desc ? [{ id: columnId, desc: true }] : [{ id: columnId, desc: false }];

		sorting = newSorting;
		onSortChange(columnId, desc ? 'desc' : 'asc');
	}

	// Convert column filters to backend format
	function convertFiltersToBackend(filters: ColumnFiltersState): FilterField[] {
		return filters.map((filter) => ({
			field: filter.id,
			value: Array.isArray(filter.value) ? filter.value.join(',') : String(filter.value)
		}));
	}

	// Track previous filter state to prevent infinite loops

	let previousFilterString = $state(JSON.stringify(convertFiltersToBackend(columnFilters)));

	// Handle filter changes with debouncing and loop prevention
	let filterTimeout: any;
	$effect(() => {
		const filterFields = convertFiltersToBackend(columnFilters);
		const currentFilterString = JSON.stringify(filterFields);

		// Only trigger if filters actually changed
		if (currentFilterString !== previousFilterString) {
			previousFilterString = currentFilterString;

			// Debounce filter changes to avoid rapid API calls
			clearTimeout(filterTimeout);
			filterTimeout = setTimeout(() => {
				onFilterChange(filterFields);
			}, 300);
		}
	});
	const columns: ColumnDef<EntryLog>[] = [
		{
			accessorKey: 'member_name',
			header: ({ column }) => {
				return renderSnippet(ColumnHeader, {
					column,
					title: 'Member',
					onSort: handleSort
				});
			},
			cell: ({ row }) => {
				const memberSnippet = createRawSnippet<[string]>((getName) => {
					const name = getName();
					return {
						render: () => `<div>${name || ''}</div>`
					};
				});

				return renderSnippet(memberSnippet, row.getValue('member_name'));
			},
			enableSorting: true,
			enableHiding: false
		},
		{
			accessorKey: 'card_id',
			header: ({ column }) =>
				renderSnippet(ColumnHeader, {
					column,
					title: 'Card ID',
					onSort: handleSort
				}),
			cell: ({ row }) => {
				return renderSnippet(CardCell, {
					value: row.original.card_id || 'N/A'
				});
			},
			enableSorting: false,
			enableHiding: false
		},
		{
			accessorKey: 'status',
			header: ({ column }) =>
				renderSnippet(ColumnHeader, {
					column,
					title: 'Status',
					onSort: handleSort
				}),
			cell: ({ row }) => {
				return renderSnippet(StatusCell, {
					value: row.original.status || 'N/A'
				});
			},
			enableSorting: true,
			enableHiding: false
		},
		{
			accessorKey: 'entry_time',
			header: ({ column }) =>
				renderSnippet(ColumnHeader, {
					column,
					title: 'Entry Time',
					onSort: handleSort
				}),
			cell: ({ row }) => {
				return renderSnippet(TimeCell, {
					value: row.original.entry_time || 'N/A'
				});
			},
			enableSorting: true,
			enableHiding: false
		},
		{
			accessorKey: 'membership_type_name',
			header: ({ column }) => {
				return renderSnippet(ColumnHeader, {
					column,
					title: 'Membership',
					onSort: handleSort,
					class: 'hidden xl:table-cell'
				});
			},
			cell: ({ row }) => {
				const snippet = createRawSnippet<[string]>((getMembership) => {
					const value = getMembership();
					return {
						render: () => `<span class='hidden xl:table-cell'>${value || ''}</span>`
					};
				});

				return renderSnippet(snippet, row.getValue('membership_type_name'));
			},
			enableSorting: false,
			enableHiding: true
		},

		{
			accessorKey: 'notes',
			header: ({ column }) => {
				return renderSnippet(ColumnHeader, {
					column,
					title: 'Details',
					onSort: handleSort,
					class: 'hidden xl:table-cell'
				});
			},
			cell: ({ row }) => {
				const snippet = createRawSnippet<[string]>((getNotes) => {
					const value = getNotes();
					return {
						render: () =>
							`<div class='text-wrap max-w-[300px] hidden xl:table-cell'>${value || ''}</div>`
					};
				});

				return renderSnippet(snippet, row.getValue('notes'));
			},
			enableSorting: false,
			enableHiding: true
		},
		{
			id: 'actions',

			header: ({ column }) =>
				renderSnippet(ColumnHeader, {
					column,
					title: 'Actions',
					onSort: handleSort,
					class: 'text-right mr-5'
				}),
			cell: ({ row }) => renderSnippet(RowActions, { row })
		}
	];

	const table = createSvelteTable({
		get data() {
			return serverData.data;
		},
		state: {
			get sorting() {
				return sorting;
			},
			get columnVisibility() {
				return columnVisibility;
			},
			get rowSelection() {
				return rowSelection;
			},
			get columnFilters() {
				return columnFilters;
			},
			get pagination() {
				return pagination;
			}
		},
		columns,
		// Disable client-side operations
		manualPagination: true,
		manualSorting: true,
		manualFiltering: true,
		// Set total row count for pagination
		rowCount: serverData.total,
		enableRowSelection: false,
		onRowSelectionChange: (updater) => {
			if (typeof updater === 'function') {
				rowSelection = updater(rowSelection);
			} else {
				rowSelection = updater;
			}
		},
		onSortingChange: (updater) => {
			if (typeof updater === 'function') {
				sorting = updater(sorting);
			} else {
				sorting = updater;
			}
		},
		onColumnFiltersChange: (updater) => {
			if (typeof updater === 'function') {
				columnFilters = updater(columnFilters);
			} else {
				columnFilters = updater;
			}
		},
		onColumnVisibilityChange: (updater) => {
			if (typeof updater === 'function') {
				columnVisibility = updater(columnVisibility);
			} else {
				columnVisibility = updater;
			}
		},
		onPaginationChange: (updater) => {
			const newPagination = typeof updater === 'function' ? updater(pagination) : updater;

			// Handle page changes
			if (newPagination.pageIndex !== pagination.pageIndex) {
				onPageChange(newPagination.pageIndex + 1); // Convert to 1-based
			}

			// Handle page size changes
			if (newPagination.pageSize !== pagination.pageSize) {
				onPageSizeChange(newPagination.pageSize);
			}

			pagination = newPagination;
		},
		getCoreRowModel: getCoreRowModel()
	});

	// Expose methods for parent component
	export function getSelectedRows() {
		return table.getSelectedRowModel().rows.map((row) => row.original);
	}

	export function clearSelection() {
		rowSelection = {};
	}

	function formatDate(dateStr: string | null | undefined): string {
		if (!dateStr) return 'N/A';
		const date = parseDateTime(dateStr).toDate('UTC');
		return date.toLocaleString('bs-BA', {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		});
	}

	export function setColumnFilters(filters: ColumnFiltersState) {
		// Update filters without triggering the effect
		const filterString = JSON.stringify(convertFiltersToBackend(filters));
		previousFilterString = filterString;
		columnFilters = filters;
	}

	// Cleanup timeout on component destroy
	$effect(() => {
		return () => {
			if (filterTimeout) {
				clearTimeout(filterTimeout);
			}
		};
	});
</script>

{#snippet StatusCell({ value }: { value: string })}
	{@const status = statuses.find((status) => status.value === value)}
	{#if status}
		<div class="flex items-center">
			<Badge variant={status.value === 'allowed' ? 'default' : 'destructive'}>
				{status.label}
			</Badge>
		</div>
	{/if}
{/snippet}

{#snippet TimeCell({ value }: { value: string })}
	{@const formattedTime = formatDate(value)}
	<div>
		{formattedTime ? formattedTime : 'N/A'}
	</div>
{/snippet}

{#snippet CardCell({ value }: { value: string })}
	<div class="flex space-x-2">
		<Badge variant="outline">{value}</Badge>
	</div>
{/snippet}

{#snippet RowActions({ row }: { row: Row<EntryLog> })}
	<div class="space-x-2 flex justify-end mr-5">
		<Button
			onclick={() => handleDelete(row.getValue('id'))}
			variant="destructive"
			size="icon"
			title="Delete Entry"
		>
			<Trash2 class="h-4 w-4" />
		</Button>
	</div>
{/snippet}

{#snippet Pagination({ table }: { table: TableType<EntryLog> })}
	<div class="flex items-center justify-between px-2">
		<div class="text-muted-foreground flex-1 text-sm">
			Showing {serverData?.data?.length || 0} of {serverData?.total || 0} total rows.
		</div>
		<div class="flex items-center space-x-6 lg:space-x-8">
			<div class="flex items-center space-x-2">
				<p class="text-sm font-medium text-muted-foreground">Rows per page</p>
				<Select.Root
					allowDeselect={false}
					type="single"
					value={`${pagination?.pageSize || 0}`}
					onValueChange={(value) => {
						onPageSizeChange(Number(value));
					}}
				>
					<Select.Trigger class="h-8 w-[70px]">
						{String(pagination.pageSize || 10)}
					</Select.Trigger>
					<Select.Content side="top">
						{#each [10, 20, 30, 40, 50] as pageSize (pageSize)}
							<Select.Item value={`${pageSize}`}>
								{pageSize}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div
				class="flex w-[100px] items-center justify-center text-sm font-medium text-muted-foreground"
			>
				Page {pagination?.pageIndex + 1 || 1} of {serverData.total_pages || 1}
			</div>
			<div class="flex items-center space-x-2">
				<Button
					variant="outline"
					class="hidden size-8 p-0 lg:flex"
					onclick={() => onPageChange(1)}
					disabled={pagination.pageIndex === 0 || loading}
				>
					<span class="sr-only">Go to first page</span>
					<ChevronsLeftIcon />
				</Button>
				<Button
					variant="outline"
					class="size-8 p-0"
					onclick={() => onPageChange(pagination.pageIndex)}
					disabled={pagination.pageIndex === 0 || loading}
				>
					<span class="sr-only">Go to previous page</span>
					<ChevronLeftIcon />
				</Button>
				<Button
					variant="outline"
					class="size-8 p-0"
					onclick={() => onPageChange(pagination.pageIndex + 2)}
					disabled={pagination.pageIndex >= serverData.total_pages - 1 || loading}
				>
					<span class="sr-only">Go to next page</span>
					<ChevronRightIcon />
				</Button>
				<Button
					variant="outline"
					class="hidden size-8 p-0 lg:flex"
					onclick={() => onPageChange(serverData.total_pages)}
					disabled={pagination.pageIndex >= serverData.total_pages - 1 || loading}
				>
					<span class="sr-only">Go to last page</span>
					<ChevronsRightIcon />
				</Button>
			</div>
		</div>
	</div>
{/snippet}

{#snippet ColumnHeader({
	column,
	title,
	class: className,
	onSort,
	...restProps
}: {
	column: Column<EntryLog>;
	title: string;
	onSort: (columnId: string, desc: boolean) => void;
} & HTMLAttributes<HTMLDivElement>)}
	{#if !column?.getCanSort()}
		<div class={cn('text-xs', className)} {...restProps}>
			{title}
		</div>
	{:else}
		<div
			class={cn('flex items-center text-xs space-x-2', className)}
			{...restProps}
			onclick={() => {
				onSort(column.id, column.getIsSorted() === 'desc' ? false : true);
			}}
		>
			<span>
				{title}
			</span>
			{#if column.getIsSorted() === 'desc'}
				<ArrowDownIcon class="text-muted-foreground/70 mr-2 size-3.5" />
			{:else if column.getIsSorted() === 'asc'}
				<ArrowUpIcon class="text-muted-foreground/70 mr-2 size-3.5" />
			{:else}
				<ChevronsUpDownIcon class="text-muted-foreground/70 mr-2 size-3.5" />
			{/if}
		</div>
	{/if}
{/snippet}

<div class="space-y-4">
	<DataTableToolbar {table} {onSearchChange} {onStartDateChange} {onEndDateChange} />
	<div class="rounded-md border">
		<Table.Root>
			<Table.Header>
				{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
					<Table.Row>
						{#each headerGroup.headers as header (header.id)}
							<Table.Head colspan={header.colSpan}>
								{#if !header.isPlaceholder}
									<FlexRender
										content={header.column.columnDef.header}
										context={header.getContext()}
									/>
								{/if}
							</Table.Head>
						{/each}
					</Table.Row>
				{/each}
			</Table.Header>
			<Table.Body>
				{#if loading}
					<Table.Row>
						<Table.Cell colspan={columns.length} class="h-24 text-center">
							<div class="flex items-center justify-center">
								<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900"></div>
								<span class="ml-2">Loading...</span>
							</div>
						</Table.Cell>
					</Table.Row>
				{:else if serverData.data?.length === 0}
					<Table.Row>
						<Table.Cell colspan={columns.length} class="h-24 text-center">No results.</Table.Cell>
					</Table.Row>
				{:else}
					{#each table.getRowModel().rows as row (row.id)}
						<Table.Row data-state={row.getIsSelected() && 'selected'}>
							{#each row.getVisibleCells() as cell (cell.id)}
								<Table.Cell>
									<FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
								</Table.Cell>
							{/each}
						</Table.Row>
					{/each}
				{/if}
			</Table.Body>
		</Table.Root>
	</div>
	{@render Pagination({ table })}
</div>
