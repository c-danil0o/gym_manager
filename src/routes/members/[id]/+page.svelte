<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import Input from '$lib/components/ui/input/input.svelte';
	import { DateFormatter } from '@internationalized/date';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import type { MembershipInfo, MemberWithMembership } from '$lib/models/member_with_membership';
	import { onMount } from 'svelte';
	import Label from '$lib/components/ui/label/label.svelte';
	import { getMembershipStatusBadgeVariant, getSubtleStatusClasses } from '$lib/utils';
	import Button from '$lib/components/ui/button/button.svelte';
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import { Pencil, Plus, Trash2 } from 'lucide-svelte';

	let isLoading = $state(false);
	let isLoadingHistory = $state(true);
	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);

	let data: MemberWithMembership | null = $state(null);
	let membershipHistory: MembershipInfo[] = $state([]);

	async function fetchMemberWithMembership() {
		isLoading = true;
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
			error = e?.message;
			toast.error(error || 'Failed to load member data.');
		} finally {
			isLoading = false;
		}
	}

	async function fetchMemberships() {
		isLoadingHistory = true;
		error = null;
		try {
			const result = await invoke<MembershipInfo[]>('get_all_memberships_for_member', {
				id: Number(memberId)
			});
			if (result) {
				membershipHistory = result;
			}
		} catch (e: any) {
			console.error('Error fetching membership history:', e);
			error = e?.message;
			toast.error(error || 'Failed to load member data.');
		} finally {
			isLoadingHistory = false;
		}
	}

	async function handleEditMember(memberId: number | undefined) {
		if (memberId) await goto(`/members/${memberId}/edit`);
	}

	async function handleEditMembership(membershipId: number | null) {
		if (memberId && membershipId) await goto(`/members/${memberId}/edit-membership?membershipId=${membershipId}`);
	}
	async function handleDeleteMembership(id: number | null) {
		if (!id) return;

		try {
			await invoke('delete_membership', { id });
			toast.success('Membership deleted successfully.');
			fetchMemberships();
			fetchMemberWithMembership();
		} catch (e: any) {
			console.error('Error deleting membership:', e);
			toast.error(e?.message || 'Failed to delete membership.');
		}
	}

	async function handleAddNewMembership(memberId: number | undefined) {
		if (memberId) await goto(`/members/${memberId}/new-membership`);
	}

	const df = new DateFormatter('bs-BA', {
		dateStyle: 'long'
	});

	onMount(async () => {
		if (memberId) {
			fetchMemberWithMembership();
			fetchMemberships();
		}
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-7xl">
	<div class="space-y-10 w-full">
		<div class="grid grid-cols-1 xl:grid-cols-2 gap-6 xl:gap-10">
			<Card.Root class="w-full">
				<Card.Header>
					<Card.Title class="text-2xl flex justify-between"
						>Member

						<Button
							on:click={() => handleEditMember(data?.id)}
							variant="outline"
							size="icon"
							title="Edit Member"
						>
							<Pencil class="h-4 w-4" />
						</Button>
					</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="space-y-6">
						<div class="w-full space-y-2">
							<Label class="font-semibold">First Name</Label>
							<Input readonly type="text" value={data?.first_name || ''} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">Last Name</Label>
							<Input readonly type="text" value={data?.last_name || ''} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">Email</Label>
							<Input readonly type="text" value={data?.email || 'Not provided'} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">Phone</Label>
							<Input readonly type="text" value={data?.phone || 'Not provided'} />
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">Date of birth</Label>
							<Input
								readonly
								type="text"
								value={data?.date_of_birth
									? df.format(new Date(data.date_of_birth))
									: 'Not provided'}
							/>
						</div>

						<div class="w-full space-y-2">
							<Label class="font-semibold">Card Number</Label>
							<Input readonly type="text" value={data?.card_id} />
						</div>
					</div>
				</Card.Content>
			</Card.Root>

			<Card.Root class="w-full">
				<Card.Header>
					<Card.Title class="text-2xl flex justify-between"
						>Membership

						{#if data?.membership_id}
						<Button
							on:click={() => handleAddNewMembership(data?.id)}
							size="icon"
							title="Add Membership"
						>
							<Plus class="h-4 w-4" />
						</Button>
						{/if}
					</Card.Title>
				</Card.Header>
				<Card.Content class="h-full">
					{#if data?.membership_id}
						<div class="space-y-6">
							<div class="w-full space-y-2">
								<Label class="font-semibold">Membership Type</Label>
								<Input type="text" readonly value={data?.membership_type_name ?? ''} />
							</div>
							<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">Duration (days)</Label>
									<Input type="text" readonly value={data?.membership_type_duration_days ?? ''} />
								</div>
								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">Visit Limit</Label>
									<Input type="text" readonly value={data?.membership_type_visit_limit ?? ''} />
								</div>

								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">Enter by (hours)</Label>
									<Input type="text" readonly value={data?.membership_type_enter_by ?? ''} />
								</div>
							</div>

							<div class="w-full space-y-2 pb-2">
								<Label class="font-semibold">Price</Label>
								<Input type="text" readonly value={data?.membership_type_price ?? ''} />
							</div>

							<Separator />

							<div class="flex flex-col md:flex-row gap-4 w-full justify-between pt-2">
								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">Start Date</Label>
									<Input
										type="text"
										readonly
										value={data?.membership_start_date
											? df.format(new Date(data.membership_start_date))
											: 'Not started yet'}
									/>
								</div>

								<div class="w-1/2 space-y-2">
									<Label class="font-semibold">End Date</Label>
									<Input
										type="text"
										readonly
										value={data?.membership_end_date
											? df.format(new Date(data.membership_end_date))
											: 'Not defined'}
									/>
								</div>
							</div>

							<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
								<div class="w-3/4 space-y-2">
									<Label class="font-semibold">Status</Label>
									<Input
										type="text"
										class={getSubtleStatusClasses(data?.membership_status || '')}
										readonly
										value={data?.membership_status}
									/>
								</div>

								<div class="w-1/4 space-y-2">
									<Label class="font-semibold">Remaining Visits</Label>
									<Input type="text" readonly value={data?.membership_remaining_visits} />
								</div>
							</div>

							<div class="w-full space-y-2 pb-2">
								<Label class="font-semibold">Purchase Date</Label>
								<Input
									type="text"
									readonly
									value={data?.membership_purchase_date
										? df.format(new Date(data.membership_purchase_date))
										: 'Membership not purchased yet'}
								/>
							</div>
						</div>
					{:else}
						<div class="flex flex-col h-full items-center justify-center space-y-4">
							<Button on:click={() => handleAddNewMembership(data?.id)} title="Edit Member"
								>Assign Membership</Button
							>
							<p class="text-muted-foreground">No membership assigned yet.</p>
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>

		<Card.Root class="mt-6">
			<Card.Header>
				<Card.Title>Membership History</Card.Title>
			</Card.Header>
			<Card.Content>
				{#if isLoadingHistory}
					<p>Loading history...</p>
					<!-- Skeleton loaders -->
				{:else if membershipHistory.length === 0}
					<p class="text-muted-foreground">No past or future memberships recorded.</p>
				{:else}
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>Type</Table.Head>
								<Table.Head>Status</Table.Head>
								<Table.Head>Start Date</Table.Head>
								<Table.Head>End Date</Table.Head>
								<Table.Head>Visits Left</Table.Head>
								<Table.Head class="text-center">Actions</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each membershipHistory as item (item.membership_id)}
								<Table.Row
									class={item.membership_status === 'active'
										? 'bg-green-50 dark:bg-green-900/30'
										: item.membership_status === 'pending'
											? 'bg-yellow-50 dark:bg-yellow-900/30'
											: ''}
								>
									<Table.Cell class="font-medium"
										>{item.membership_type_name || 'Unknown Type'}</Table.Cell
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
											on:click={() => handleEditMembership(item.membership_id)}
											variant="outline"
											size="icon"
											title="Edit Member"
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
													<AlertDialog.Title>Are you absolutely sure?</AlertDialog.Title>
													<AlertDialog.Description>
														This action cannot be undone. This will permanently delete membership
														from this member!</AlertDialog.Description
													>
												</AlertDialog.Header>
												<AlertDialog.Footer>
													<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
													<AlertDialog.Action
														on:click={() => handleDeleteMembership(item.membership_id)}
														>Continue</AlertDialog.Action
													>
												</AlertDialog.Footer>
											</AlertDialog.Content>
										</AlertDialog.Root>
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				{/if}
			</Card.Content>
		</Card.Root>
	</div>
</div>
