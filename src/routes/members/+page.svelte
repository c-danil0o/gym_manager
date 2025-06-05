<script lang="ts">
	import { goto } from '$app/navigation';
	import { MemberDataTable } from '$lib/components/member-table';
	import type { MemberInfo } from '$lib/models/member_with_membership';
	import type { FilterField, QueryRequest, QueryResponse } from '$lib/models/table-state';
	import { m } from '$lib/paraglide/messages';
	import { setHeader } from '$lib/stores/state';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	onMount(() => {
		setHeader({
			title: m['common.members'](),
			showBackButton: false
		});
	});
	let tableData = $state<QueryResponse<MemberInfo>>({
		data: [],
		total: 0,
		page: 1,
		per_page: 10,
		total_pages: 0
	});

	let loading = $state(false);
	let currentParams = $state<QueryRequest>({
		page: 1,
		per_page: 10,
		order_by: undefined,
		order_direction: undefined,
		search_string: '',
		filter_fields: []
	});

	// Reference to the table component for accessing methods
	let tableRef: any;

	// Fetch data from backend
	async function fetchTableData(params: QueryRequest) {
		loading = true;
		try {
			const response = await invoke<QueryResponse<MemberInfo>>(
				'get_members_with_memberships_paginated',
				{
					payload: {
						page: params.page,
						per_page: params.per_page,
						order_by: params.order_by,
						order_direction: params.order_direction,
						search_string: params.search_string || '',
						filter_fields: params.filter_fields || []
					}
				}
			);

			tableData = response;
		} catch (error) {
			console.error('Failed to fetch table data:', error);
			toast.error(m.failed_to_fetch_members());
			// Handle error appropriately
		} finally {
			loading = false;
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
	function handleEditMember(memberId: number) {
		goto(`/members/${memberId}/edit`);
	}
	function handleRenewMembership(memberId: number, membershipId: number | null) {
		if (membershipId) goto(`/members/${memberId}/renew-membership?membershipId=${membershipId}`);
	}
	function handleViewMember(memberId: number) {
		goto(`/members/${memberId}`);
	}

	function handleAddMember() {
		goto('/members/new');
	}

	// Load initial data
	$effect(() => {
		fetchTableData(currentParams);
	});
</script>

<div>
	<MemberDataTable
		bind:this={tableRef}
		serverData={tableData}
		{loading}
		onPageChange={handlePageChange}
		onPageSizeChange={handlePageSizeChange}
		onSortChange={handleSortChange}
		onSearchChange={debouncedSearchChange}
		onFilterChange={handleFilterChange}
		{handleEditMember}
		{handleRenewMembership}
		{handleViewMember}
		{handleAddMember}
	/>
</div>
