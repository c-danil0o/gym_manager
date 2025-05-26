<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { newMemberSchema, type NewMemberTypeSchema } from '$lib/schemas/new_member_schema';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import DateField from '$lib/components/date-field/date-field.svelte';

	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';

	let submitting = false;

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
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('add_member', { payload: result.data });
				toast.success('New member added successfully!');
				handleCancel();
			} else {
				toast.error('Data is not valid!');
			}
		} catch (error) {
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

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">Add New Member</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit} class="space-y-6">
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

				<div class="flex gap-20 justify-around">
					<Form.Button variant="outline" on:click={handleCancel} class="w-full">Cancel</Form.Button>
					<Form.Button type="submit" class="w-full">Confirm</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
