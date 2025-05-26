<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import {
		membershipTypeSchema,
		type MembershipTypeSchema
	} from '$lib/schemas/membership_type_schema';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import Textarea from '$lib/components/ui/textarea/textarea.svelte';

	let submitting = false;

	const initialValues: z.infer<MembershipTypeSchema> = {
		name: '',
		duration_days: null,
		visit_limit: null,
		enter_by: null,
		price: 0,
		description: ''
	};

	const form = superForm(initialValues, {
		validators: zodClient(membershipTypeSchema),
		syncFlashMessage: true,
		dataType: 'json',
		SPA: true,
		taintedMessage: null,
		onUpdated({ form: currentForm }) {
			if (!currentForm.valid) console.log('Client errors:', currentForm.errors);
		}
	});

	const { form: formData, enhance } = form;

	async function handleSubmit() {
		submitting = true;
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('add_membership_type', { payload: result.data });
				toast.success('New membership type added successfully!');
				handleCancel();
			} else {
				toast.error('Data is not valid!');
			}
		} catch (error) {
			toast.error('Failed to add new membership type!');
			submitting = false;
			return;
		} finally {
			submitting = false;
		}
	}
	async function handleCancel() {
		await goto('/member-ships');
	}
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">Add New Membership Type</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit} class="space-y-6">
				<Form.Field {form} name="name">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Name</Form.Label>
						<Input {...attrs} type="text" bind:value={$formData.name} />
						<Form.FieldErrors />
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="duration_days">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Duration</Form.Label>
						<Input {...attrs} type="number" bind:value={$formData.duration_days} />
						<Form.FieldErrors />
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="visit_limit">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Visit limit</Form.Label>
						<Input {...attrs} type="number" bind:value={$formData.visit_limit} />
						<Form.FieldErrors />
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="enter_by">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Enter by (hour of the day)</Form.Label>
						<Input
							{...attrs}
							type="number"
							placeholder="optional"
							bind:value={$formData.enter_by}
						/>
						<Form.FieldErrors />
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="price">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Price</Form.Label>
						<Input {...attrs} type="number" bind:value={$formData.price} />
						<Form.FieldErrors />
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="description">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Description</Form.Label>
						<Textarea {...attrs} bind:value={$formData.description} />
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
