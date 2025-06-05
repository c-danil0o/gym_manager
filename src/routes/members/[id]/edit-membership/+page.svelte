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
	import { cn, getSubtleStatusClasses, translateStatus } from '$lib/utils';
	import { membershipSchema, type MembershipSchemaType } from '$lib/schemas/membership_schema';
	import type { MembershipInfo } from '$lib/models/member_with_membership';
	import Button from '$lib/components/ui/button/button.svelte';
	import Checkbox from '$lib/components/ui/checkbox/checkbox.svelte';
	import { setHeader, setLoading } from '$lib/stores/state';
	import type { ErrorResponse } from '$lib/models/error';
	import { m } from '$lib/paraglide/messages';

	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);
	const membershipId = $derived(page.url.searchParams.get('membershipId'));

	let membershipTypes = $state<MembershipType[]>([]);
	let selectedMembershipType: MembershipType | null = $state(null);
	let membership_status: string | null = $state(null);
	let memberData = $state<{ first_name: string | null; last_name: string | null } | null>(null);

	async function fetchMembershipTypes() {
		error = null;
		try {
			const result = await invoke<MembershipType[]>('get_all_membership_types');
			membershipTypes = result || [];
		} catch (e: any) {
			console.error('Error fetching membership types:', e);
			error = e?.message;
			toast.error(m.toast_failed_membership_types());
		}
	}
	async function fetchMembership() {
		error = null;
		try {
			const result = await invoke<MembershipInfo>('get_membership_by_id', {
				id: Number(membershipId)
			});
			if (result) {
				$formData.member_id = result.member_id;
				$formData.membership_id = result.membership_id;
				$formData.membership_type_id = result.membership_type_id;
				$formData.membership_start_date = result.membership_start_date;
				$formData.membership_end_date = result.membership_end_date;
				$formData.membership_remaining_visits = result.membership_remaining_visits;
				$formData.membership_suspended = result?.membership_status === 'suspended';

				selectedMembershipType =
					membershipTypes.find((t) => t.id === result.membership_type_id) || null;

				membership_status = result.membership_status;
				memberData = {
					first_name: result.member_first_name,
					last_name: result.member_last_name
				};
			} else {
				toast.error(m.membership_not_found());
				window.history.back();
			}
		} catch (e: any) {
			console.error('Error fetching membership status:', e);
			error = e?.message;
			toast.error(m.failed_load_membership());
		}
	}

	const initialValues: z.infer<MembershipSchemaType> = {
		member_id: 0,
		membership_id: 0,
		membership_type_id: null,
		membership_start_date: null,
		membership_end_date: null,
		membership_remaining_visits: 0,
		membership_suspended: false
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
		setLoading(true);
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('save_membership', {
					payload: result.data
				});
				toast.success(m.memebership_update_success());
				window.history.back();
			} else {
				toast.error(m['toast_error_invalid_data']());
			}
		} catch (error) {
			console.log(error);
			// TRANSLATE ERROR
			const errorMessage = (error as ErrorResponse)?.message || 'Failed to edit membership!';
			toast.error(errorMessage);

			return;
		} finally {
			setLoading(false);
		}
	}

	async function handleCancel() {
		window.history.back();
	}

	const df = new DateFormatter('bs-BA', {
		dateStyle: 'long'
	});

	let start_date = $state<DateValue | undefined>();
	let end_date = $state<DateValue | undefined>();

	$effect(() => {
		start_date = $formData.membership_start_date
			? parseDate($formData.membership_start_date)
			: undefined;
	});

	$effect(() => {
		end_date = $formData.membership_end_date ? parseDate($formData.membership_end_date) : undefined;
	});

	function onChangeStartDate(newValue: DateValue | undefined) {
		$formData.membership_start_date = newValue ? newValue.toString() : null;
		const durationDays = selectedMembershipType?.duration_days;
		if (newValue && durationDays) {
			const endDate = newValue.add({ days: durationDays });
			$formData.membership_end_date = endDate.toString();
		}
	}

	function onChangeEndDate(newValue: DateValue | undefined) {
		$formData.membership_end_date = newValue ? newValue.toString() : null;
	}

	function onMembershipTypeChange(id: number) {
		if (Number.isNaN(id)) return;
		$formData.membership_type_id = id;
		selectedMembershipType = membershipTypes.find((t) => t.id === id) || null;
		if (!selectedMembershipType) return;

		let start = $formData.membership_start_date;
		if (!start) {
			start = today(getLocalTimeZone()).toString();
			$formData.membership_start_date = start;
		}

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
			title: m.edit_membership(),
			showBackButton: true
		});
		setLoading(true);
		await fetchMembershipTypes();
		if (membershipId) {
			await fetchMembership();
		}
		setLoading(false);
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root class="w-full">
		<Card.Header>
			<Card.Title class="text-2xl">{m['common.membership']()}</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-10 w-full">
				<div class="space-y-6">
					<div class="w-full space-y-2">
						<Label class="font-semibold">{m['common.member']}</Label>
						<Input
							type="text"
							readonly
							value={memberData?.first_name && memberData?.last_name
								? memberData?.first_name + ' ' + memberData?.last_name
								: ''}
						/>
					</div>
					<Form.Field {form} name="membership_type_id">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.membership_type()}</Form.Label>
								<Select.Root
									type="single"
									value={$formData.membership_type_id ? String($formData.membership_type_id) : ''}
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
											: m.select_membership_type()}
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
													{m.no_types_available()}
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
							<Label class="font-semibold">{m.duration_days()}</Label>
							<Input type="text" readonly value={selectedMembershipType?.duration_days ?? ''} />
						</div>
						<div class="w-1/2 space-y-2">
							<Label class="font-semibold">{m.visit_limit()}</Label>
							<Input type="text" readonly value={selectedMembershipType?.visit_limit ?? ''} />
						</div>

						<div class="w-1/2 space-y-2">
							<Label class="font-semibold">{m.enter_by_hours()}</Label>
							<Input type="text" readonly value={selectedMembershipType?.enter_by ?? ''} />
						</div>
					</div>

					<div class="w-full space-y-2 pb-2">
						<Label class="font-semibold">{m.price()}</Label>
						<Input type="text" readonly value={selectedMembershipType?.price ?? ''} />
					</div>

					<Separator />

					<div class="flex flex-col md:flex-row gap-4 w-full justify-between pt-2">
						<Form.Field {form} name="membership_start_date" class="w-1/2">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">{m.start_date()}</Form.Label>
									<Popover.Root>
										<Popover.Trigger
											class={cn(
												buttonVariants({ variant: 'outline' }),
												'w-full justify-start pl-4 text-left font-normal',
												!start_date && 'text-muted-foreground'
											)}
										>
											{start_date
												? df.format(start_date.toDate(getLocalTimeZone()))
												: m.pick_date()}
											<CalendarIcon class="ml-auto size-4 opacity-50" />
										</Popover.Trigger>
										<Popover.Content class="w-auto p-0" side="top">
											<Calendar
												type="single"
												value={start_date}
												onValueChange={onChangeStartDate}
											/>
										</Popover.Content>
									</Popover.Root>
									<Form.FieldErrors />
								{/snippet}
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="membership_end_date" class="w-1/2">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">{m.end_date()}</Form.Label>
									<Popover.Root>
										<Popover.Trigger
											class={cn(
												buttonVariants({ variant: 'outline' }),
												'w-full justify-start pl-4 text-left font-normal',
												!end_date && 'text-muted-foreground'
											)}
										>
											{end_date ? df.format(end_date.toDate(getLocalTimeZone())) : m.pick_date()}
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
						<div class="w-1/2 space-y-2 pb-2">
							<Label class="font-semibold">{m.status()}</Label>
							<Input
								type="text"
								class={getSubtleStatusClasses(membership_status || '')}
								readonly
								value={translateStatus(membership_status)}
							/>
						</div>

						<Form.Field {form} name="membership_remaining_visits" class="w-1/4">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">{m.remaining_visits()}</Form.Label>
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

						<Form.Field {form} name="membership_suspended" class="w-1/4">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold w-full text-center">{m.membership_suspended()}</Form.Label>
									<div class="flex items-center h-[30px] w-full justify-center">
										<Checkbox {...props} bind:checked={$formData.membership_suspended} />
									</div>
									<Form.FieldErrors />
								{/snippet}
							</Form.Control>
						</Form.Field>
					</div>

					<div class="w-full space-y-2 pb-2">
						<Label class="font-semibold">{m.purchase_date()}</Label>
						<Input type="text" readonly value={df.format(new Date())} />
					</div>
				</div>

				<div class="flex gap-20 justify-around">
					<Button variant="outline" onclick={handleCancel} class="w-full">{m.cancel()}</Button>
					<Form.Button type="submit" class="w-full">{m.save()}</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
