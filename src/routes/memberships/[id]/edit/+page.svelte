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
	import Button from '$lib/components/ui/button/button.svelte';
	import { setHeader } from '$lib/stores/state';
	import { onMount } from 'svelte';
	import type { ErrorResponse } from '$lib/models/error';
	import type { MembershipType } from '$lib/models/membership_type';
	import { page } from '$app/state';

	let submitting = false;
	let isLoading = $state(false);
	let error: string | null = $state(null);

	const membershipTypeId = $derived(page.params.id);

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
		resetForm: false,
		onUpdated({ form: currentForm }) {
			if (!currentForm.valid) console.log('Client errors:', currentForm.errors);
		}
	});

	const { form: formData, enhance } = form;

	async function fetchMembershipType() {
		isLoading = true;
		if (!membershipTypeId) {
			toast.error('Error loading membership type.');
			isLoading = false;
			return;
		}

		try {
			const result = await invoke<MembershipType>('get_membership_type_by_id', {
				id: Number(membershipTypeId)
			});
			if (result) {
				formData.set({
					name: result.name,
					duration_days: result.duration_days,
					visit_limit: result.visit_limit,
					enter_by: result.enter_by,
					price: result.price,
					description: result.description || ''
				});
			} else {
				toast.error('Membership type not found.');
				goto('/memberships');
				return;
			}
		} catch (e: any) {
			console.error('Error fetching member data:', e);
			error = e?.message;
			toast.error(error || 'Failed to load member data.');
		} finally {
			isLoading = false;
		}
	}

	async function handleSubmit() {
		submitting = true;
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('update_membership_type', {
					id: Number(membershipTypeId),
					payload: result.data
				});
				toast.success('Membership type updated successfully!');
				handleCancel();
			} else {
				toast.error('Data is not valid!');
			}
		} catch (error) {
			console.log(error);
			const errorMessage = (error as ErrorResponse)?.message || 'Failed to update membership type!';
			toast.error(errorMessage);
			submitting = false;
			return;
		} finally {
			submitting = false;
		}
	}
	async function handleCancel() {
		await goto('/memberships');
	}
	onMount(() => {
		setHeader({
			title: 'Update Membership Type',
			showBackButton: true
		});
		fetchMembershipType();
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">Update Membership Type</Card.Title>
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
						<Input {...attrs} type="number" min="1" bind:value={$formData.duration_days} />
						<Form.FieldErrors />
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="visit_limit">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Visit limit</Form.Label>
						<Input {...attrs} type="number" min="0" max={$formData.duration_days} bind:value={$formData.visit_limit} />
						<Form.FieldErrors />
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="enter_by">
					<Form.Control let:attrs>
						<Form.Label class="font-semibold">Enter by (hour of the day)</Form.Label>
						<Input
							{...attrs}
							type="number"
							min="0"
							max="23"
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
					<Button variant="outline" on:click={handleCancel} class="w-full">Cancel</Button>
					<Form.Button type="submit" class="w-full">Save</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
