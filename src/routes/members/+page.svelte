<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	// Shadcn UI Imports
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import Input from '$lib/components/ui/input/input.svelte';
	import * as Pagination from '$lib/components/ui/pagination'; // Requires `npx shadcn-svelte@latest add pagination`
	import { Badge } from '$lib/components/ui/badge'; // Requires `npx shadcn-svelte@latest add badge`
	import { Skeleton } from '$lib/components/ui/skeleton'; // Requires `npx shadcn-svelte@latest add skeleton`

	// Icons
	import PlusCircle from 'lucide-svelte/icons/plus-circle';
	import Search from 'lucide-svelte/icons/search';
	import Pencil from 'lucide-svelte/icons/pencil';
	import type { MemberInfo, PaginatedMembersResponse } from '$lib/models/member_with_membership';

	let membersData = $state<MemberInfo[]>([]);
	let totalItems = $state(0);
	let totalPages = $state(1);
	let currentPage = $state(1);
	let pageSize = $state(20);

	let isLoading = $state(true);
	let error = $state<string | null>(null);

	let searchQuery = $state('');
	let debouncedSearchQuery = $state(''); // For debouncing search input

	// --- Data Fetching ---
	async function fetchMembers(pageToFetch = currentPage, query = debouncedSearchQuery) {
		isLoading = true;
		error = null;
		try {
			const payload = {
				page: pageToFetch,
				page_size: pageSize,
				search_query: query || null // Send null if empty
			};
			console.log('Fetching members with payload:', payload);
			const result = await invoke<PaginatedMembersResponse>(
				'get_members_with_memberships_paginated',
				{ payload }
			);
			membersData = result.members;
			totalItems = result.total_items;
			totalPages = result.total_pages;
			currentPage = result.current_page;
			pageSize = result.page_size;
			console.log('Fetched data:', result);
		} catch (e: unknown) {
			console.error('Error fetching members:', e);
			error = e instanceof Error ? e.message : 'An unknown error occurred';
			toast.error(error || 'Failed to load members.');
			membersData = []; // Clear data on error
			totalItems = 0;
			totalPages = 1;
		} finally {
			isLoading = false;
		}
	}

	// --- Search Debouncing ---
	let searchTimeout: number;
	function handleSearchInput() {
		clearTimeout(searchTimeout);
		searchTimeout = window.setTimeout(() => {
			debouncedSearchQuery = searchQuery.trim();
			fetchMembers(1, debouncedSearchQuery);
		}, 500);
	}

	onMount(() => {
		fetchMembers();
	});

	function handleAddNewMember() {
		goto('/members/new');
	}

	function handleEditMember(memberId: number) {
		goto(`/members/${memberId}/edit`);
	}

	// Placeholder for delete if you implement it
	// async function handleDeleteMember(memberId: number, memberName: string) {
	//     // ... confirmation dialog ...
	//     // ... invoke tauri command ...
	//     // ... toast message ...
	//     // ... fetchMembers(currentPage, debouncedSearchQuery); ...
	// }

	function getMembershipStatusBadgeVariant(
		status: string | null
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status?.toLowerCase()) {
			case 'active':
				return 'default'; // Or a success color if you have one
			case 'expired':
				return 'destructive';
			case 'pending':
				return 'secondary';
			case 'cancelled':
				return 'outline';
			default:
				return 'secondary';
		}
	}
</script>

<div class="space-y-6">
	<div class="flex flex-col md:flex-row items-center justify-between gap-4">
		<h1 class="text-2xl font-semibold">Members Overview</h1>
		<div class="flex w-full md:w-auto items-center gap-2">
			<div class="relative w-full md:w-64">
				<Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
				<Input
					type="search"
					placeholder="Search by name..."
					class="pl-8 w-full"
					bind:value={searchQuery}
					on:input={handleSearchInput}
				/>
			</div>
			<Button on:click={handleAddNewMember} class="w-full md:w-auto">
				<PlusCircle class="mr-2 h-4 w-4" />
				Add Member
			</Button>
		</div>
	</div>

	{#if isLoading && membersData.length === 0}
		<!-- Show skeletons only on initial load -->
		<Card.Root>
			<Card.Content class="pt-6">
				{#each Array(5) as _}
					<div class="flex items-center space-x-4 py-3 border-b last:border-b-0">
						<Skeleton class="h-10 w-10 rounded-full" />
						<div class="space-y-2 flex-grow">
							<Skeleton class="h-4 w-3/4" />
							<Skeleton class="h-4 w-1/2" />
						</div>
						<Skeleton class="h-8 w-20" />
					</div>
				{/each}
			</Card.Content>
		</Card.Root>
	{:else if error}
		<Card.Root class="border-destructive">
			<Card.Header
				><Card.Title class="text-destructive">Error Loading Members</Card.Title></Card.Header
			>
			<Card.Content>
				<p>{error}</p>
				<Button
					on:click={() => fetchMembers(currentPage, debouncedSearchQuery)}
					variant="outline"
					class="mt-4">Try Again</Button
				>
			</Card.Content>
		</Card.Root>
	{:else if membersData.length === 0 && !isLoading}
		<Card.Root>
			<Card.Content class="pt-6 text-center">
				<p class="text-muted-foreground">
					{#if debouncedSearchQuery}
						No members found matching "{debouncedSearchQuery}".
					{:else}
						No members found.
					{/if}
				</p>
				{#if !debouncedSearchQuery}
					<Button on:click={handleAddNewMember} variant="link" class="mt-2"
						>Add the first member!</Button
					>
				{/if}
			</Card.Content>
		</Card.Root>
	{:else}
		<Card.Root>
			{#if isLoading && membersData.length > 0}
				<div class="absolute inset-0 bg-background/50 flex items-center justify-center z-10">
					<p>Loading...</p>
					<!-- Replace with a spinner -->
				</div>
			{/if}
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Name</Table.Head>
						<Table.Head class="hidden md:table-cell">Card ID</Table.Head>
						<Table.Head class="hidden lg:table-cell">Email</Table.Head>
						<Table.Head>Membership</Table.Head>
						<Table.Head class="hidden md:table-cell">Status</Table.Head>
						<Table.Head class="text-center">Actions</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each membersData as member (member.id)}
						<Table.Row on:click={() => handleEditMember(member.id)} class="cursor-pointer">
							<Table.Cell class="font-medium">{member.first_name} {member.last_name}</Table.Cell>
							<Table.Cell class="hidden md:table-cell text-muted-foreground"
								>{member.card_id || 'N/A'}</Table.Cell
							>
							<Table.Cell class="hidden lg:table-cell text-muted-foreground"
								>{member.email || 'N/A'}</Table.Cell
							>
							<Table.Cell>
								{#if member.membership_type_name}
									{member.membership_type_name}
									{#if member.membership_end_date}
										<span class="text-xs text-muted-foreground block">
											Ends: {new Date(member.membership_end_date).toLocaleDateString()}
										</span>
									{/if}
								{:else}
									<span class="text-muted-foreground">None</span>
								{/if}
							</Table.Cell>
							<Table.Cell class="hidden md:table-cell">
								{#if member.membership_status}
									<Badge variant={getMembershipStatusBadgeVariant(member.membership_status)}>
										{member.membership_status}
									</Badge>
								{:else}
									<span class="text-muted-foreground">-</span>
								{/if}
							</Table.Cell>
							<Table.Cell class="text-center">
								<Button
									on:click={() => handleEditMember(member.id)}
									variant="outline"
									size="icon"
									title="Edit Member"
								>
									<Pencil class="h-4 w-4" />
								</Button>
								<!-- Optional Delete Button -->
								<!--
                                <Button on:click={() => handleDeleteMember(member.id, `${member.first_name} ${member.last_name}`)} variant="ghost" size="icon" title="Delete Member" class="text-destructive hover:text-destructive-foreground hover:bg-destructive/80 ml-2">
                                    <Trash2 class="h-4 w-4" />
                                </Button>
                                -->
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Card.Root>

		{#if totalPages}
			<div class="mt-6 flex justify-center">
				<Pagination.Root
					count={totalItems}
					page={currentPage}
					perPage={pageSize}
					onPageChange={(e) => fetchMembers(e)}
					let:pages
					let:currentPage
				>
					<Pagination.Content>
						<Pagination.Item>
							<Pagination.PrevButton />
						</Pagination.Item>
						{#each pages as page (page.key)}
							{#if page.type === 'ellipsis'}
								<Pagination.Item>
									<Pagination.Ellipsis />
								</Pagination.Item>
							{:else}
								<Pagination.Item>
									<Pagination.Link {page} isActive={currentPage === page.value}>
										{page.value}
									</Pagination.Link>
								</Pagination.Item>
							{/if}
						{/each}
						<Pagination.Item>
							<Pagination.NextButton />
						</Pagination.Item>
					</Pagination.Content>
				</Pagination.Root>
			</div>
		{/if}
	{/if}
</div>
