<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import * as Form from '$lib/components/ui/form/index.js';
	import * as RadioGroup from '$lib/components/ui/radio-group/index.js';
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
	import { getSubtleStatusClasses, translateErrorCode, translateStatus } from '$lib/utils';
	import { membershipSchema, type MembershipSchemaType } from '$lib/schemas/membership_schema';
	import type { MembershipInfo } from '$lib/models/member_with_membership';
	import Button from '$lib/components/ui/button/button.svelte';
	import { setHeader, setLoading } from '$lib/stores/state';
	import type { ErrorResponse } from '$lib/models/error';
	import { m } from '$lib/paraglide/messages';
	import DatePicker from '$lib/components/date-picker/date-picker.svelte';

	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);
	const membershipId = $derived(page.url.searchParams.get('membershipId'));

	let membershipTypes = $state<MembershipType[]>([]);
	let filteredMembershipTypes = $state<MembershipType[]>([]);
	let selectedMembershipType: MembershipType | null = $state(null);
	let membership_status: string | null = $state(null);
	let membershipData = $state<MembershipInfo | null>(null);
	let renewType: 'end' | 'now' = $state('end');
	let endRenewDisabled = $state(false);

	async function fetchMembershipTypes() {
		error = null;
		try {
			const result = await invoke<MembershipType[]>('get_all_membership_types');
			membershipTypes = result || [];
			filteredMembershipTypes = membershipTypes.filter((t) => t.is_active);
		} catch (e: any) {
			console.error('Error fetching membership types:', e);
			toast.error(m.toast_failed_membership_types());
		} finally {
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
				$formData.membership_type_id = result.membership_type_id;
				if (result?.membership_end_date) {
					const endDate = parseDate(result.membership_end_date);
					$formData.membership_start_date = endDate.add({ days: 1 }).toString();
					if (endDate.compare(today(getLocalTimeZone())) < 0) {
						renewType = 'now';
						endRenewDisabled = true;
					}
				}
				selectedMembershipType =
					membershipTypes.find((t) => t.id === result.membership_type_id) || null;
				membershipData = result;
				if (result?.membership_type_id) onMembershipTypeChange(result.membership_type_id);
			} else {
				toast.error(m.membership_type_not_found());
				window.history.back();
			}
		} catch (e: any) {
			console.error('Error fetching membership data:', e);
			toast.error(m.failed_load_membership());
		}
	}
	$effect(() => {
		const currentRenewType = renewType;

		if (currentRenewType === 'end') {
			if (membershipData?.membership_end_date) {
				const newStartDate = parseDate(membershipData.membership_end_date).add({ days: 1 });
				$formData.membership_start_date = newStartDate.toString();
				$formData.membership_end_date = newStartDate
					.add({ days: selectedMembershipType?.duration_days || 0 })
					.toString();
			}
		} else {
			$formData.membership_start_date = today(getLocalTimeZone()).toString();
			$formData.membership_end_date = today(getLocalTimeZone())
				.add({ days: selectedMembershipType?.duration_days || 0 })
				.toString();
		}
	});

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
		setLoading(true);
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('save_membership', {
					payload: result.data
				});
				toast.success(m.membership_renew_success());
				window.history.back();
			} else {
				toast.error(m.toast_error_invalid_data());
			}
		} catch (error) {
			console.log(error);
			const errorMessage = error as ErrorResponse;
			if (errorMessage?.error_code && errorMessage?.params) {
				toast.error(translateErrorCode(errorMessage.error_code, errorMessage.params));
			} else {
				toast.error(m.toast_renew_membership_fail());
			}
		} finally {
			setLoading(false);
		}
	}

	async function handleCancel() {
		window.history.back();
	}
	const locale = m.locale_code() || 'bs-BA';

	const df = new DateFormatter(locale, {
		dateStyle: 'long'
	});

	const df2 = new DateFormatter(locale, {
		year: 'numeric',
		month: '2-digit',
		day: '2-digit'
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
		if (selectedMembershipType?.visit_limit && selectedMembershipType.visit_limit > 0) {
			$formData.membership_remaining_visits = selectedMembershipType.visit_limit;
		} else {
			$formData.membership_remaining_visits = selectedMembershipType.duration_days || 0;
		}
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
		setLoading(true);
		setHeader({
			title: m.renew_membership(),
			showBackButton: true
		});
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
			<Card.Title class="text-xl">{m.old_membership()}</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-10 w-full">
				<div class="space-y-6">
					<div class="w-full space-y-2">
						<Label class="font-semibold">{m['common.member']()}</Label>
						<Input
							type="text"
							readonly
							value={membershipData?.member_first_name && membershipData?.member_last_name
								? membershipData?.member_first_name + ' ' + membershipData?.member_last_name
								: ''}
						/>
					</div>

					<div class="w-full space-y-2 pb-2">
						<Label class="font-semibold">{m.membership_type()}</Label>
						<Input
							type="text"
							readonly
							value={membershipTypes.find((t) => t.id === membershipData?.membership_type_id)
								?.name ?? ''}
						/>
					</div>

					<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
						<div class="w-1/2 space-y-2 pb-2">
							<Label class="font-semibold">{m.expiry_date()}</Label>
							<Input
								type="text"
								readonly
								value={membershipData?.membership_end_date
									? df.format(new Date(membershipData.membership_end_date))
									: 'Not found'}
							/>
						</div>
						<div class="w-1/2 space-y-2 pb-2">
							<Label class="font-semibold">{m.status()}</Label>
							<Input
								type="text"
								class={getSubtleStatusClasses(membershipData?.membership_status || '')}
								readonly
								value={translateStatus(membershipData?.membership_status || '')}
							/>
						</div>
					</div>

					<Separator />

					<Card.Title class="text-xl">{m.new_membership()}</Card.Title>

					<Form.Field {form} name="membership_type_id">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.membership_type()}</Form.Label>
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
											: m.select_membership_type()}
									</Select.Trigger>
									<Select.Content>
										<Select.Group>
											{#each filteredMembershipTypes as type (type.id)}
												<Select.Item value={String(type.id)} label={type.name}
													>{type.name}</Select.Item
												>
											{/each}
											{#if filteredMembershipTypes.length === 0}
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
							<div class="relative flex">
								<Input
									type="text"
									readonly
									class="pr-15"
									value={selectedMembershipType?.enter_by ?? ''}
								/>
								<span
									class="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground pointer-events-none text-xs"
								>
									h
								</span>
							</div>
						</div>
					</div>

					<div class="w-full space-y-2 pb-2">
						<Label class="font-semibold">{m.price()}</Label>
						<div class="relative flex">
							<Input
								type="text"
								class="pr-15"
								readonly
								value={selectedMembershipType?.price ?? ''}
							/>
							<span
								class="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground pointer-events-none text-xs"
							>
								{m.locale_currency()}
							</span>
						</div>
					</div>
					<RadioGroup.Root bind:value={renewType} class="flex justify-evenly" name="type">
						<div class="flex items-center space-x-3 space-y-0">
							<RadioGroup.Item value="end" disabled={endRenewDisabled} />
							<Label class="font-normal">{m.from_end_date()}</Label>
						</div>
						<div class="flex items-center space-x-3 space-y-0">
							<RadioGroup.Item
								value="now"
								disabled={membershipData?.membership_status === 'active' ||
									membershipData?.membership_status === 'suspended'}
							/>
							<Label class="font-normal">{m.from_now()}</Label>
						</div>
					</RadioGroup.Root>
					<div class="flex flex-col md:flex-row gap-4 w-full justify-between pt-2">
						<Form.Field {form} name="membership_start_date" class="w-1/2">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">{m.start_date()}</Form.Label>
									<Input
										type="text"
										readonly
										value={df2.format(new Date($formData?.membership_start_date || new Date()))}
									/>
									<Form.FieldErrors />
								{/snippet}
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="membership_end_date" class="w-1/2">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label class="font-semibold">{m.end_date()}</Form.Label>
									<DatePicker
										{...props}
										value={end_date}
										onValueChange={onChangeEndDate}
										minValue={$formData.membership_start_date
											? parseDate($formData.membership_start_date).add({
													days: selectedMembershipType?.duration_days || 0
												})
											: today(getLocalTimeZone())}
										{locale}
										height="h-9 py-1.5"
									/>
									<Form.FieldErrors />
								{/snippet}
							</Form.Control>
						</Form.Field>
					</div>

					<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
						<div class="w-2/3 space-y-2 pb-2">
							<Label class="font-semibold">{m.status()}</Label>
							<Input
								type="text"
								class={getSubtleStatusClasses(membership_status || '')}
								readonly
								value={translateStatus(membership_status)}
							/>
						</div>

						<Form.Field {form} name="membership_remaining_visits" class="w-1/3">
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
