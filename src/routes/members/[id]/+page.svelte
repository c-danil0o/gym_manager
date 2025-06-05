<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Pagination from '$lib/components/ui/pagination';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import Input from '$lib/components/ui/input/input.svelte';
	import { DateFormatter } from '@internationalized/date';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import type { MembershipInfo, MemberWithMembership } from '$lib/models/member_with_membership';
	import { onMount } from 'svelte';
	import Label from '$lib/components/ui/label/label.svelte';
	import { getMembershipStatusBadgeVariant, getSubtleStatusClasses, translateStatus } from '$lib/utils';
	import Button from '$lib/components/ui/button/button.svelte';
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import { Pencil, Plus, Trash2, RefreshCcw } from 'lucide-svelte';
	import { setHeader, setLoading } from '$lib/stores/state';
	import type { QueryResponse } from '$lib/models/table-state';
	import { m } from '$lib/paraglide/messages';

	let isLoadingHistory = $state(true);
	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);

	let data: MemberWithMembership | null = $state(null);

	let membershipParams = $state({
		page: 1,
		perPage: 10,
		totalPages: 1,
		totalItems: 0
	});

	async function fetchMemberWithMembership() {
		error = null;
		try {
			const result = await invoke<MemberWithMembership>('get_member_by_id_with_membership', {
				payload: {
					id: Number(memberId)
				}
			});
			if (result) {
				data = result;
			}
		} catch (e: any) {
			console.error('Error fetching member with membership:', e);
			toast.error(m.failed_to_load_member());
		}
	}

	let membershipHistory = $state<QueryResponse<MembershipInfo>>({
		data: [],
		total: 0,
		page: 1,
		per_page: 10,
		total_pages: 0
	});

	async function fetchMemberships() {
		isLoadingHistory = true;
		error = null;
		try {
			const result = await invoke<QueryResponse<MembershipInfo>>('get_all_memberships_for_member', {
				id: Number(memberId),
				payload: {
					page: membershipParams.page,
					per_page: membershipParams.perPage
				}
			});
			if (result) {
				membershipHistory = result;
				membershipParams.totalItems = result.total;
				membershipParams.totalPages = Math.ceil(result.total / membershipParams.perPage);
			}
		} catch (e: any) {
			console.error('Error fetching membership history:', e);
			toast.error(m.failed_load_membership_data());
		} finally {
			isLoadingHistory = false;
		}
	}

	$effect(() => {
		const currentPage = membershipParams.page;
		const perPage = membershipParams.perPage;
		fetchMemberships();
	});

	async function handleEditMember(memberId: number | undefined) {
		if (memberId) await goto(`/members/${memberId}/edit`);
	}

	async function handleRenewMembership(membershipId: number | null | undefined) {
		if (membershipId)
			await goto(`/members/${memberId}/renew-membership?membershipId=${membershipId}`);
	}

	async function handleDeleteMember(id: number | null) {
		if (!id) return;

		try {
			await invoke('delete_member', { id });
			toast.success(m.member_delete_success());
			goto('/members');
		} catch (e: any) {
			console.error('Error deleting member:', e);
			toast.error(m.member_delete_fail());
		}
	}

	async function handleEditMembership(membershipId: number | null) {
		if (memberId && membershipId)
			await goto(`/members/${memberId}/edit-membership?membershipId=${membershipId}`);
	}
	async function handleDeleteMembership(id: number | null) {
		if (!id) return;

		setLoading(true);
		try {
			await invoke('delete_membership', { id });
			toast.success(m.membership_delete_success());
			fetchMemberships();
			fetchMemberWithMembership();
		} catch (e: any) {
			console.error('Error deleting membership:', e);
			toast.error(m.membership_delete_fail());
		} finally {
			setLoading(false);
		}
	}

	async function handleAddNewMembership(memberId: number | undefined) {
		if (memberId) await goto(`/members/${memberId}/new-membership`);
	}

	const df = new DateFormatter('bs-BA', {
		dateStyle: 'long'
	});

	onMount(async () => {
		setHeader({
			title: m.member_details(),
			showBackButton: true
		});
		if (memberId) {
			setLoading(true);
			await Promise.all([fetchMemberWithMembership(), fetchMemberships()]);
		}
		setLoading(false);
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-7xl">
	<div class="space-y-10 w-full">
		<div class="grid grid-cols-1 xl:grid-cols-2 gap-6 xl:gap-10">
			<Card.Root class="w-full">
				<Card.Header>
					<Card.Title class="text-2xl flex justify-between"
						>{m['common.member']()}

						<div class="space-x-2">
							<Button
								onclick={() => handleEditMember(data?.id)}
								variant="outline"
								size="icon"
								title=m.edit_member()
							>
								<Pencil class="h-4 w-4" />
							</Button>

							<AlertDialog.Root>
								<AlertDialog.Trigger>
									<Button variant="destructive" size="icon" title={m['common.delete']()}>
										<Trash2 class="h-4 w-4" />
									</Button>
								</AlertDialog.Trigger>
								<AlertDialog.Content>
									<AlertDialog.Header>
										<AlertDialog.Title>{m['common.are_you_sure']()}</AlertDialog.Title>
										<AlertDialog.Description>
											{m.member_delete_desc()}</AlertDialog.Description
										>
									</AlertDialog.Header>
									<AlertDialog.Footer>
										<AlertDialog.Cancel>{m['common.cancel']()}</AlertDialog.Cancel>
										<AlertDialog.Action onclick={() => handleDeleteMember(data?.id || null)}
											>{m['common.confirm']()}</AlertDialog.Action
										>
									</AlertDialog.Footer>
								</AlertDialog.Content>
							</AlertDialog.Root>
						</div>
					</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="space-y-6">
						<div class="w-full space-y-2">
							<Label class="font-semibold">{m.first_name()}</Label>
							<Input readonly type="text" value={data?.first_name || ''} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">{m.last_name()}</Label>
							<Input readonly type="text" value={data?.last_name || ''} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">{m.email()}</Label>
							<Input readonly type="text" value={data?.email || 'Not provided'} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">{m.phone()}</Label>
							<Input readonly type="text" value={data?.phone || 'Not provided'} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">{m.date_of_birth()}</Label>
							<Input
								readonly
								type="text"
								value={data?.date_of_birth
									? df.format(new Date(data.date_of_birth))
									: 'Not provided'}
							/>
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">{m.card_number()}</Label>
							<Input readonly type="text" value={data?.card_id} />
						</div>
					</div>
				</Card.Content>
			</Card.Root>

			<Card.Root class="w-full">
				<Card.Header>
					<Card.Title class="text-2xl flex justify-between"
						>{m['common.membership']()}

						{#if data?.membership_id}
							<div class="space-x-2">
								<Button
									onclick={() => handleRenewMembership(data?.membership_id)}
									variant="outline"
									size="icon"
									class="bg-blue-100 dark:bg-blue-900"
									disabled={data?.membership_status !== 'active' &&
										data?.membership_status !== 'expired'}
									title={m.renew_membership()}
								>
									<RefreshCcw class="h-4 w-4" />
								</Button>
								<Button
									onclick={() => handleAddNewMembership(data?.id)}
									size="icon"
									title={m.add_membership()}
								>
									<Plus class="h-4 w-4" />
								</Button>
							</div>
						{/if}
					</Card.Title>
				</Card.Header>
				<Card.Content class="h-full">
					{#if data?.membership_id}
						<div class="space-y-6">
							<div class="w-full space-y-2">
								<Label class="font-semibold">{m.membership_type()}</Label>
								<Input type="text" readonly value={data?.membership_type_name ?? ''} />
							</div>
							<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">{m.duration_days()}</Label>
									<Input type="text" readonly value={data?.membership_type_duration_days ?? ''} />
								</div>
								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">{m['common.visit_limit']()}</Label>
									<Input type="text" readonly value={data?.membership_type_visit_limit ?? ''} />
								</div>

								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">{m.enter_by_hours()}</Label>
									<Input type="text" readonly value={data?.membership_type_enter_by ?? ''} />
								</div>
							</div>

							<div class="w-full space-y-2 pb-2">
								<Label class="font-semibold">{m['common.price']()}</Label>
								<Input type="text" readonly value={data?.membership_type_price ?? ''} />
							</div>

							<Separator />

							<div class="flex flex-col md:flex-row gap-4 w-full justify-between pt-2">
								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">{m.start_date()}</Label>
									<Input
										type="text"
										readonly
										value={data?.membership_start_date
											? df.format(new Date(data.membership_start_date))
											: m.not_started_yet()}
									/>
								</div>

								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">{m.end_date()}</Label>
									<Input
										type="text"
										readonly
										value={data?.membership_end_date
											? df.format(new Date(data.membership_end_date))
											: m.not_defined()}
									/>
								</div>
							</div>

							<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
								<div class="w-3/4 space-y-2">
									<Label class="font-semibold">{m['common.status']()}</Label>
									<Input
										type="text"
										class={getSubtleStatusClasses(data?.membership_status || '')}
										readonly
										value={translateStatus(data?.membership_status)}
									/>
								</div>

								<div class="w-1/4 space-y-2">
									<Label class="font-semibold">{m.remaining_visits()}</Label>
									<Input type="text" readonly value={data?.membership_remaining_visits} />
								</div>
							</div>

							<div class="w-full space-y-2 pb-2">
								<Label class="font-semibold">{m.purchase_date()}</Label>
								<Input
									type="text"
									readonly
									value={data?.membership_purchase_date
										? df.format(new Date(data.membership_purchase_date))
										: m.memership_not_purchased()}
								/>
							</div>
						</div>
					{:else}
						<div class="flex flex-col h-full items-center justify-center space-y-4">
							<Button onclick={() => handleAddNewMembership(data?.id)} title="Edit Member"
								>{m.assign_membership()}</Button
							>
							<p class="text-muted-foreground">{m.no_membership_yet()}</p>
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>

		<Card.Root class="mt-6">
			<Card.Header>
				<Card.Title>{m.membership_history()}</Card.Title>
			</Card.Header>
			<Card.Content>
				{#if isLoadingHistory}
					<p>{m.loading_history()}</p>
					<!-- Skeleton loaders -->
				{:else if membershipHistory.data.length === 0}
					<p class="text-muted-foreground">{m.no_past_or_future_memberships()}</p>
				{:else}
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>{m.type()}</Table.Head>
								<Table.Head>{m.status()}</Table.Head>
								<Table.Head>{m.start_date()}</Table.Head>
								<Table.Head>{m.end_date()}</Table.Head>
								<Table.Head>{m.visits_left()}</Table.Head>
								<Table.Head class="text-center">{m.actions()}</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each membershipHistory.data as item (item.membership_id)}
								<Table.Row
									class={item.membership_status === 'active'
										? 'bg-green-50 dark:bg-green-900/30'
										: item.membership_status === 'pending'
											? 'bg-yellow-50 dark:bg-yellow-900/30'
											: ''}
								>
									<Table.Cell class="font-medium"
										>{item.membership_type_name || {m.deleted_type()}}</Table.Cell
									>
									<Table.Cell>
										<Badge variant={getMembershipStatusBadgeVariant(item.membership_status)}
											>{item.membership_status || 'N/A'}</Badge
										>
									</Table.Cell>
									<Table.Cell
										>{item?.membership_start_date
											? df.format(new Date(item.membership_start_date))
											: ''}</Table.Cell
									>
									<Table.Cell
										>{item?.membership_end_date
											? df.format(new Date(item.membership_end_date))
											: ''}</Table.Cell
									>
									<Table.Cell>{item?.membership_remaining_visits ?? 'N/A'}</Table.Cell>

									<Table.Cell class="text-center space-x-2">
										<Button
											onclick={() => handleEditMembership(item.membership_id)}
											variant="outline"
											size="icon"
											disabled={item.membership_status !== 'active' &&
												item.membership_status !== 'pending'}
											title={m.edit_membership()}
										>
											<Pencil class="h-4 w-4" />
										</Button>

										<AlertDialog.Root>
											<AlertDialog.Trigger>
												<Button variant="destructive" size="icon" title="Delete">
													<Trash2 class="h-4 w-4" />
												</Button>
											</AlertDialog.Trigger>
											<AlertDialog.Content>
												<AlertDialog.Header>
													<AlertDialog.Title>{m['common.are_you_sure']()}</AlertDialog.Title>
													<AlertDialog.Description>
														{m.delete_membership_desc()}</AlertDialog.Description
													>
												</AlertDialog.Header>
												<AlertDialog.Footer>
													<AlertDialog.Cancel>{m['common.cancel']()}</AlertDialog.Cancel>
													<AlertDialog.Action
														onclick={() => handleDeleteMembership(item.membership_id)}
														>{m['common.confirm']()}</AlertDialog.Action
													>
												</AlertDialog.Footer>
											</AlertDialog.Content>
										</AlertDialog.Root>
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
					<Pagination.Root
						class="mt-6"
						count={membershipParams.totalItems}
						perPage={membershipParams.perPage}
						bind:page={membershipParams.page}
					>
						{#snippet children({ pages, currentPage })}
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
						{/snippet}
					</Pagination.Root>
				{/if}
			</Card.Content>
		</Card.Root>
	</div>
</div>
