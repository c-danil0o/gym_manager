<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import { buttonVariants } from '$lib/components/ui/button/index.js';
	import { Calendar } from '$lib/components/ui/calendar/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import {
		DateFormatter,
		getLocalTimeZone,
		parseDate,
		today,
		type DateValue
	} from '@internationalized/date';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import type { MembershipType } from '$lib/models/membership_type';
	import { onMount } from 'svelte';
	import Label from '$lib/components/ui/label/label.svelte';
	import { CalendarIcon } from 'lucide-svelte';
	import { cn, getSubtleStatusClasses } from '$lib/utils';
	import { membershipSchema, type MembershipSchemaType } from '$lib/schemas/membership_schema';
	import type { MembershipInfo } from '$lib/models/member_with_membership';
	import Button from '$lib/components/ui/button/button.svelte';
	import { setHeader } from '$lib/stores/state';
	import type { ErrorResponse } from '$lib/models/error';

	let isLoading = $state(false);
	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);
	const membershipId = $derived(page.url.searchParams.get('membershipId'));

	let membershipTypes = $state<MembershipType[]>([]);
	let selectedMembershipType: MembershipType | null = $state(null);
	let membership_status: string | null = $state(null);
	let membershipData = $state<MembershipInfo | null>(null);

	async function fetchMembershipTypes() {
		isLoading = true;
		error = null;
		try {
			const result = await invoke<MembershipType[]>('get_all_membership_types');
			membershipTypes = result || [];
		} catch (e: any) {
			console.error('Error fetching membership types:', e);
			error = e?.message;
			toast.error(error || 'Failed to load membership types.');
		} finally {
			isLoading = false;
		}
	}
	async function fetchMembership() {
		isLoading = true;
		error = null;
		try {
			const result = await invoke<MembershipInfo>('get_membership_by_id', {
				id: Number(membershipId)
			});
			if (result) {
				$formData.member_id = result.member_id;
				$formData.membership_type_id = result.membership_type_id;
				if (result?.membership_end_date)
					$formData.membership_start_date = parseDate(result.membership_end_date)
						.add({ days: 1 })
						.toString();
				selectedMembershipType =
					membershipTypes.find((t) => t.id === result.membership_type_id) || null;
				membershipData = result;
				if (result?.membership_type_id) onMembershipTypeChange(result.membership_type_id);
			} else {
				toast.error('Membership not found.');
				window.history.back();
			}
		} catch (e: any) {
			console.error('Error fetching membership data:', e);
			error = e?.message;
			toast.error(error || 'Failed to load membership status.');
		} finally {
			isLoading = false;
		}
	}

	const initialValues: z.infer<MembershipSchemaType> = {
		member_id: 0,
		membership_type_id: null,
		membership_start_date: null,
		membership_end_date: null,
		membership_remaining_visits: 0
	};

	const form = superForm(initialValues, {
		validators: zodClient(membershipSchema),
		syncFlashMessage: true,
		dataType: 'json',
		SPA: true,
		taintedMessage: null,
		resetForm: false,
		onUpdated({ form: currentForm }) {
			if (!currentForm.valid) console.log('Client errors:', currentForm.errors);
		}
	});

	const { form: formData, enhance } = form;

	$effect(() => {
		if (memberId) {
			$formData.member_id = Number(memberId);
		}
	});

	async function handleSubmit() {
		isLoading = true;
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('save_membership', {
					payload: result.data
				});
				toast.success('Membership renewed successfully!');
				window.history.back();
			} else {
				toast.error('Data is not valid!');
			}
		} catch (error) {
			console.log(error);
			const errorMessage = (error as ErrorResponse)?.message || 'Failed to renew membership!';
			toast.error(errorMessage);

			return;
		} finally {
			isLoading = false;
		}
	}

	async function handleCancel() {
		window.history.back();
	}

	const df = new DateFormatter('bs-BA', {
		dateStyle: 'long'
	});

	let end_date = $state<DateValue | undefined>();

	$effect(() => {
		end_date = $formData.membership_end_date ? parseDate($formData.membership_end_date) : undefined;
	});

	function onChangeEndDate(newValue: DateValue | undefined) {
		$formData.membership_end_date = newValue ? newValue.toString() : null;
	}

	function onMembershipTypeChange(id: number) {
		if (Number.isNaN(id)) return;
		$formData.membership_type_id = id;
		selectedMembershipType = membershipTypes.find((t) => t.id === id) || null;
		if (!selectedMembershipType) return;

		let start = $formData.membership_start_date;
		if (!start) return;

		if (selectedMembershipType.duration_days) {
			const startDateObj = parseDate(start);
			const newEnd = startDateObj.add({ days: selectedMembershipType.duration_days }).toString();
			$formData.membership_end_date = newEnd;
		}
		if (selectedMembershipType.visit_limit)
			$formData.membership_remaining_visits = selectedMembershipType.visit_limit;
	}

	$effect(() => {
		// Get current values from signals to establish dependencies
		const currentStartDate = $formData.membership_start_date;
		const currentEndDate = $formData.membership_end_date;
		const currentRemainingVisits = $formData.membership_remaining_visits;
		const currentlySuspended = $formData.membership_suspended;

		let newStatus = 'inactive';

		if (currentlySuspended) {
			newStatus = 'suspended';
		} else {
			if (!currentStartDate || !currentEndDate) {
				newStatus = 'inactive';
			} else {
				if (
					currentRemainingVisits !== null &&
					currentRemainingVisits !== undefined &&
					currentRemainingVisits <= 0
				) {
					newStatus = 'expired';
				} else {
					const todayDate = today(getLocalTimeZone());
					const startDateVal = parseDate(currentStartDate);
					const endDateVal = parseDate(currentEndDate);

					if (startDateVal.compare(todayDate) > 0) {
						newStatus = 'pending';
					} else {
						if (endDateVal.compare(todayDate) < 0) {
							newStatus = 'expired';
						} else {
							newStatus = 'active';
						}
					}
				}
			}
		}

		if (membership_status !== newStatus) {
			membership_status = newStatus;
		}
	});

	onMount(async () => {
		setHeader({
			title: 'Renew Membership',
			showBackButton: true
		});
		await fetchMembershipTypes();
		if (membershipId) {
			fetchMembership();
		}
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root class="w-full">
		<Card.Header>
			<Card.Title class="text-xl">Old Membership</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-10 w-full">
				<div class="space-y-6">
					<div class="w-full space-y-2">
						<Label class="font-semibold">Member</Label>
						<Input
							type="text"
							readonly
							value={membershipData?.member_first_name && membershipData?.member_last_name
								? membershipData?.member_first_name + ' ' + membershipData?.member_last_name
								: ''}
						/>
					</div>

					<div class="w-full space-y-2 pb-2">
						<Label class="font-semibold">Membership Type</Label>
						<Input
							type="text"
							readonly
							value={membershipTypes.find((t) => t.id === membershipData?.membership_type_id)
								?.name ?? ''}
						/>
					</div>

					<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
						<div class="w-1/2 space-y-2 pb-2">
							<Label class="font-semibold">Expiry Date</Label>
							<Input
								type="text"
								readonly
								value={membershipData?.membership_end_date
									? df.format(new Date(membershipData.membership_end_date))
									: 'Not found'}
							/>
						</div>
						<div class="w-1/2 space-y-2 pb-2">
							<Label class="font-semibold">Status</Label>
							<Input
								type="text"
								class={getSubtleStatusClasses(membershipData?.membership_status || '')}
								readonly
								value={membershipData?.membership_status}
							/>
						</div>
					</div>

					<Separator />

					<Card.Title class="text-xl">New Membership</Card.Title>

					<Form.Field {form} name="membership_type_id">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">Membership Type</Form.Label>
								<Select.Root
									type="single"
									value={String($formData.membership_type_id || '')}
									onValueChange={(v: any) => {
										if (v) {
											const numValue = Number(v);
											onMembershipTypeChange(numValue);
										} else {
											$formData.membership_type_id = null;
										}
									}}
								>
									<Select.Trigger {...props}>
										{selectedMembershipType
											? selectedMembershipType.name
											: 'Select membership type'}
									</Select.Trigger>
									<Select.Content>
										<Select.Group>
											{#each membershipTypes as type (type.id)}
												<Select.Item value={String(type.id)} label={type.name}
													>{type.name}</Select.Item
												>
											{/each}
											{#if membershipTypes.length === 0 && !isLoading}
												<div class="px-2 py-1.5 text-sm text-muted-foreground">
													No types available.
												</div>
											{/if}
										</Select.Group>
									</Select.Content>
								</Select.Root>
								<Form.FieldErrors />
							{/snippet}
						</Form.Control>
					</Form.Field>
					<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
						<div class="w-1/2 space-y-2">
							<Label class="font-semibold">Duration (days)</Label>
							<Input type="text" readonly value={selectedMembershipType?.duration_days ?? ''} />
						</div>
						<div class="w-1/2 space-y-2">
							<Label class="font-semibold">Visit Limit</Label>
							<Input type="text" readonly value={selectedMembershipType?.visit_limit ?? ''} />
						</div>

						<div class="w-1/2 space-y-2">
							<Label class="font-semibold">Enter by (hours)</Label>
							<Input type="text" readonly value={selectedMembershipType?.enter_by ?? ''} />
						</div>
					</div>

					<div class="w-full space-y-2 pb-2">
						<Label class="font-semibold">Price</Label>
						<Input type="text" readonly value={selectedMembershipType?.price ?? ''} />
					</div>
					<div class="flex flex-col md:flex-row gap-4 w-full justify-between pt-2">
						<Form.Field {form} name="membership_start_date" class="w-1/2">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">Start Date</Form.Label>
									<Input
										type="text"
										readonly
										value={df.format(new Date($formData?.membership_start_date || new Date()))}
									/>
									<Form.FieldErrors />
								{/snippet}
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="membership_end_date" class="w-1/2">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">End Date</Form.Label>
									<Popover.Root>
										<Popover.Trigger
											class={cn(
												buttonVariants({ variant: 'outline' }),
												'w-full justify-start pl-4 text-left font-normal',
												!end_date && 'text-muted-foreground'
											)}
										>
											{end_date ? df.format(end_date.toDate(getLocalTimeZone())) : 'Pick a date'}
											<CalendarIcon class="ml-auto size-4 opacity-50" />
										</Popover.Trigger>
										<Popover.Content class="w-auto p-0" side="top">
											<Calendar type="single" value={end_date} onValueChange={onChangeEndDate} />
										</Popover.Content>
									</Popover.Root>
									<Form.FieldErrors />
								{/snippet}
							</Form.Control>
						</Form.Field>
					</div>

					<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
						<div class="w-2/3 space-y-2 pb-2">
							<Label class="font-semibold">Status</Label>
							<Input
								type="text"
								class={getSubtleStatusClasses(membership_status || '')}
								readonly
								value={membership_status}
							/>
						</div>

						<Form.Field {form} name="membership_remaining_visits" class="w-1/3">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">Remaining Visits</Form.Label>
									<Input
										{...props}
										type="number"
										min={0}
										max={selectedMembershipType?.duration_days}
										bind:value={$formData.membership_remaining_visits}
									/>
									<Form.FieldErrors />
								{/snippet}
							</Form.Control>
						</Form.Field>
					</div>

					<div class="w-full space-y-2 pb-2">
						<Label class="font-semibold">Purchase Date</Label>
						<Input type="text" readonly value={df.format(new Date())} />
					</div>
				</div>

				<div class="flex gap-20 justify-around">
					<Button variant="outline" onclick={handleCancel} class="w-full">Cancel</Button>
					<Form.Button type="submit" class="w-full">Save</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
