<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { newMemberSchema, type NewMemberTypeSchema } from '$lib/schemas/new_member_schema';
	import type { Member } from '$lib/models/member';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import DateField from '$lib/components/date-field/date-field.svelte';

	import * as Form from '$lib/components/ui/form/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import DatePicker from '$lib/components/date-picker/date-picker.svelte';

	const memberId = $derived(page.params.id);
	const selectedMembershipType = undefined;

	const initialValues: z.infer<NewMemberTypeSchema> = {
		card_id: '',
		first_name: '',
		last_name: '',
		email: '',
		phone: '',
		date_of_birth: null
	};

	const form = superForm(initialValues, {
		validators: zodClient(newMemberSchema),
		syncFlashMessage: true,
		dataType: 'json',
		SPA: true,
		taintedMessage: null,
		onUpdated({ form: currentForm }) {
			if (!currentForm.valid) console.log('Client errors:', currentForm.errors);
		}
	});

	const { form: formData, enhance } = form;

	let placeholder: DateValue = today(getLocalTimeZone());

	function handleDateChange(newValue: DateValue | undefined) {
		$formData.date_of_birth = newValue ? newValue.toString() : null;
	}

	async function handleSubmit() {
		submitting = true;
		newMember = null;
		try {
			const result = await form.validateForm();
			if (result.valid) {
				const member: Member = await invoke('add_member', { payload: result.data });
				newMember = member;
				showMembershipPrompt = true;
				toast.success('New member added successfully!');
			} else {
				toast.error('Data is not valid!');
			}
		} catch (error) {
			showMembershipPrompt = false;
			toast.error('Failed to add new member!');
			submitting = false;
			return;
		} finally {
			submitting = false;
		}
	}
	async function handleCancel() {
		await goto('/members');
	}
</script>

<div class="container mx-auto p-4 md:p-8 max-w-7xl">
	<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-10 w-full">
		<div class="grid grid-cols-1 xl:grid-cols-2 gap-6 xl:gap-10">
			<Card.Root class="w-full">
				<Card.Header>
					<Card.Title class="text-2xl">Member</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="space-y-6">
						<Form.Field {form} name="first_name">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">First Name</Form.Label>
								<Input {...attrs} type="text" bind:value={$formData.first_name} />
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="last_name">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Last Name</Form.Label>
								<Input {...attrs} type="text" bind:value={$formData.last_name} />
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="email">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Email</Form.Label>
								<Input {...attrs} type="email" bind:value={$formData.email} />
								<Form.Description class="text-xs">Optional</Form.Description>
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="phone">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Phone</Form.Label>
								<Input {...attrs} type="text" bind:value={$formData.phone} />
								<Form.Description class="text-xs">Optional</Form.Description>
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="date_of_birth">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Date of birth</Form.Label>
								<DateField
									{...attrs}
									{placeholder}
									onValueChange={handleDateChange}
									locale="bs-BA"
								/>
								<Form.FieldErrors />
								<Form.Description class="text-xs">Optional</Form.Description>
							</Form.Control>
						</Form.Field>

						<Form.Field {form} name="card_id">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Card Number</Form.Label>
								<Input {...attrs} type="text" bind:value={$formData.card_id} />
								<Form.Description class="text-xs">Use scanner or enter manually</Form.Description>
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>
					</div>
				</Card.Content>
			</Card.Root>

			<Card.Root class="w-full">
				<Card.Header>
					<Card.Title class="text-2xl">Membership</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="space-y-6">
						<Form.Field {form} name="first_name">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Membership Type</Form.Label>
								<Select.Root
									selected={selectedMembershipType}
									onSelectedChange={(v) => {
										v && ($formData.selectedMemberShipTypeId = v.value);
									}}
								>
									<Select.Trigger {...attrs}>
										<Select.Value placeholder="Select membership type" />
									</Select.Trigger>
									<Select.Content>
										<Select.Item value="m@example.com" label="FULL" />
										<Select.Item value="m@google.com" label="HALF" />
										<Select.Item value="m@support.com" label="lkj" />
									</Select.Content>
								</Select.Root>
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>
						<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
							<Form.Field {form} name="last_name">
								<Form.Control let:attrs>
									<Form.Label class="font-semibold">Duration (days)</Form.Label>
									<Input {...attrs} type="text" readonly bind:value={$formData.last_name} />
									<Form.FieldErrors />
								</Form.Control>
							</Form.Field>

							<Form.Field {form} name="email">
								<Form.Control let:attrs>
									<Form.Label class="font-semibold">Visit Limit</Form.Label>
									<Input {...attrs} type="text" readonly bind:value={$formData.email} />
									<Form.FieldErrors />
								</Form.Control>
							</Form.Field>

							<Form.Field {form} name="email">
								<Form.Control let:attrs>
									<Form.Label class="font-semibold">Entry By (hours)</Form.Label>
									<Input {...attrs} type="text" readonly bind:value={$formData.email} />
									<Form.FieldErrors />
								</Form.Control>
							</Form.Field>
						</div>

						<Form.Field {form} name="phone" class="pb-2">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Price</Form.Label>
								<Input {...attrs} type="text" readonly bind:value={$formData.phone} />
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>

						<Separator />

						<div class="flex flex-col md:flex-row gap-4 w-full justify-between pt-2">
							<Form.Field {form} name="date_of_birth" class="w-full">
								<Form.Control let:attrs>
									<Form.Label class="font-semibold">Start Date</Form.Label>
									<DatePicker {placeholder} locale="bs-BA" dateStyle="long" class="w-full" />
									<Form.FieldErrors />
								</Form.Control>
							</Form.Field>

							<Form.Field {form} name="date_of_birth" class="w-full">
								<Form.Control let:attrs>
									<Form.Label class="font-semibold">End Date</Form.Label>
									<DatePicker {placeholder} locale="bs-BA" dateStyle="long" class="w-full" />
									<Form.FieldErrors />
								</Form.Control>
							</Form.Field>
						</div>

						<div class="flex flex-col md:flex-row gap-4 w-full justify-between">
							<Form.Field {form} name="card_id" class="w-3/4">
								<Form.Control let:attrs>
									<Form.Label class="font-semibold">Status</Form.Label>
									<Input {...attrs} type="text" bind:value={$formData.card_id} />
									<Form.FieldErrors />
								</Form.Control>
							</Form.Field>

							<Form.Field {form} name="card_id" class="w-1/4">
								<Form.Control let:attrs>
									<Form.Label class="font-semibold">Remaining Visits</Form.Label>
									<Input {...attrs} type="text" bind:value={$formData.card_id} />
									<Form.FieldErrors />
								</Form.Control>
							</Form.Field>
						</div>

						<Form.Field {form} name="date_of_birth" class="w-full">
							<Form.Control let:attrs>
								<Form.Label class="font-semibold">Purchase Date</Form.Label>
								<DatePicker {placeholder} locale="bs-BA" dateStyle="long" class="w-full" />
								<Form.FieldErrors />
							</Form.Control>
						</Form.Field>
					</div>
				</Card.Content>
			</Card.Root>
		</div>

		<Card.Root class="w-1/2 mx-auto">
			<Card.Content>
				<div class="flex gap-20 justify-around">
					<Form.Button variant="outline" on:click={handleCancel} class="w-full">Cancel</Form.Button>
					<Form.Button type="submit" class="w-full">Save</Form.Button>
				</div>
			</Card.Content>
		</Card.Root>
	</form>
</div>
