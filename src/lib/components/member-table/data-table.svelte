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
	import { cn, getMembershipStatusBadgeVariant } from '$lib/utils.js';
	import type { MemberInfo } from '$lib/models/member_with_membership';
	import { Pencil, RefreshCcw } from 'lucide-svelte';

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
		serverData: TableData<MemberInfo>;
		loading?: boolean;
		onPageChange: (page: number) => void;
		onPageSizeChange: (pageSize: number) => void;
		onSortChange: (orderBy: string | null, orderDirection: 'asc' | 'desc' | null) => void;
		onSearchChange: (searchString: string) => void;
		onFilterChange: (filterFields: FilterField[]) => void;
		handleEditMember?: (memberId: number) => void;
		handleRenewMembership?: (memberId: number, membershipId: number | null) => void;
		handleViewMember?: (memberId: number) => void;
	}

	let {
		serverData = [{ data: [], total: 0, page: 1, per_page: 10, total_pages: 1 }],
		loading = false,
		onPageChange,
		onPageSizeChange,
		onSortChange,
		onSearchChange,
		onFilterChange,
		handleEditMember = () => {},
		handleViewMember = () => {},
		handleRenewMembership = () => {}
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
	const columns: ColumnDef<MemberInfo>[] = [
		{
			accessorKey: 'name',
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
						render: () => `<div>${name}</div>`
					};
				});

				return renderSnippet(memberSnippet, row.getValue('name'));
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
			enableSorting: true,
			enableHiding: false
		},
		{
			accessorKey: 'email',
			header: ({ column }) => {
				return renderSnippet(ColumnHeader, {
					column,
					title: 'Email',
					onSort: handleSort
				});
			},
			cell: ({ row }) => {
				const emailSnippet = createRawSnippet<[string]>((getEmail) => {
					const email = getEmail();
					return {
						render: () => `<span class='text-muted-foreground'>${email}</span>`
					};
				});

				return renderSnippet(emailSnippet, row.getValue('email'));
			},
			enableSorting: false,
			enableHiding: false
		},
		{
			accessorKey: 'membership_type_name',
			header: ({ column }) => {
				return renderSnippet(ColumnHeader, {
					column,
					title: 'Membership',
					onSort: handleSort
				});
			},
			cell: ({ row }) => {
				const snippet = createRawSnippet<[string]>((getMembership) => {
					const value = getMembership();
					return {
						render: () => `<span>${value || ''}</span>`
					};
				});

				return renderSnippet(snippet, row.getValue('membership_type_name'));
			},
			enableSorting: false,
			enableHiding: false
		},
		{
			accessorKey: 'membership_status',
			header: ({ column }) =>
				renderSnippet(ColumnHeader, {
					column,
					title: 'Status',
					onSort: handleSort
				}),
			cell: ({ row }) => {
				return renderSnippet(StatusCell, {
					value: row.original.membership_status || 'N/A'
				});
			},
			enableSorting: true
		},
		{
			id: 'actions',

			header: ({ column }) =>
				renderSnippet(ColumnHeader, {
					column,
					title: 'Actions',
					onSort: handleSort,
					class: 'text-right mr-10'
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
		<div class="flex w-[100px] items-center">
			<Badge variant={getMembershipStatusBadgeVariant(status.value)}>
				{status.label}
			</Badge>
		</div>
	{/if}
{/snippet}

{#snippet CardCell({ value }: { value: string })}
	<div class="flex space-x-2">
		<Badge variant="outline">{value}</Badge>
	</div>
{/snippet}

{#snippet RowActions({ row }: { row: Row<MemberInfo> })}
	<div class="space-x-2 flex justify-end mr-5">
		<Button
			onclick={(e) => {
				e.stopPropagation();
				handleRenewMembership(row.original.id, row.original.membership_id);
			}}
			variant="outline"
			class="bg-blue-100 dark:bg-blue-900"
			size="icon"
			disabled={row.getValue('membership_status') !== 'active' &&
				row.getValue('membership_status') !== 'expired'}
			title="Renew Membership"
		>
			<RefreshCcw class="h-4 w-4" />
		</Button>
		<Button
			onclick={(e) => {
				e.stopPropagation();
				handleEditMember(row.original.id);
			}}
			variant="outline"
			size="icon"
			title="Edit Member"
		>
			<Pencil class="h-4 w-4" />
		</Button>
	</div>
{/snippet}

{#snippet Pagination({ table }: { table: TableType<MemberInfo> })}
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
	column: Column<MemberInfo>;
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
	<DataTableToolbar {table} {onSearchChange} />
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
						<Table.Row
							data-state={row.getIsSelected() && 'selected'}
							onclick={() => handleViewMember(row.original.id)}
						>
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
