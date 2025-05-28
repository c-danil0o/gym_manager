<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import { DateFormatter } from '@internationalized/date';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import type { MemberWithMembership } from '$lib/models/member_with_membership';
	import { onMount, tick } from 'svelte';
	import Label from '$lib/components/ui/label/label.svelte';
	import { cn, getSubtleStatusClasses } from '$lib/utils';
	import Button from '$lib/components/ui/button/button.svelte';

	let isLoading = $state(false);
	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);

	let data: MemberWithMembership | null = $state(null);

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

	async function handleCancel() {
		await goto('/members');
	}

	const df = new DateFormatter('bs-BA', {
		dateStyle: 'long'
	});

	onMount(async () => {
		if (memberId) {
			fetchMemberWithMembership();
		}
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-7xl">
	<div class="space-y-10 w-full">
		<div class="grid grid-cols-1 xl:grid-cols-2 gap-6 xl:gap-10">
			<Card.Root class="w-full">
				<Card.Header>
					<Card.Title class="text-2xl">Member</Card.Title>
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
					<Card.Title class="text-2xl">Membership</Card.Title>
				</Card.Header>
				<Card.Content>
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
				</Card.Content>
			</Card.Root>
		</div>

		<div class="h-[700px]"></div>
		<Card.Root class="w-1/2 mx-auto">
			<Card.Content>
				<div class="flex gap-20 justify-around">
					<Button variant="outline" on:click={handleCancel} class="w-full">Cancel</Button>
					<Button type="submit" class="w-full">Save</Button>
				</div>
			</Card.Content>
		</Card.Root>
	</div>
</div>
