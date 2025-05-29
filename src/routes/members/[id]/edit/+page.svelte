<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import DateField from '$lib/components/date-field/date-field.svelte';
	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import { parseDate, type DateValue } from '@internationalized/date';
	import { onMount } from 'svelte';
	import { editMemberSchema, type EditMemberTypeSchema } from '$lib/schemas/edit_member_schema';
	import type { Member } from '$lib/models/member';
	import { setHeader } from '$lib/stores/state';
	import Button from '$lib/components/ui/button/button.svelte';
	import type { ErrorResponse } from '$lib/models/error';

	let isLoading = $state(false);
	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);

	async function fetchMember() {
		isLoading = true;
		error = null;
		try {
			const result = await invoke<Member>('get_member_by_id', {
				payload: {
					id: Number(memberId)
				}
			});
			if (result) {
				$formData.id = result.id;
				$formData.card_id = result.card_id ?? '';
				$formData.first_name = result.first_name;
				$formData.last_name = result.last_name;
				$formData.email = result.email;
				$formData.phone = result.phone;
				$formData.date_of_birth = result.date_of_birth ?? null;
			}
		} catch (e: any) {
			console.error('Error fetching member data:', e);
			error = e?.message;
			toast.error(error || 'Failed to load member data.');
		} finally {
			isLoading = false;
		}
	}

	const initialValues: z.infer<EditMemberTypeSchema> = {
		id: 0,
		card_id: '',
		first_name: '',
		last_name: '',
		email: '',
		phone: '',
		date_of_birth: null
	};

	const form = superForm(initialValues, {
		validators: zodClient(editMemberSchema),
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

	async function handleSubmit() {
		isLoading = true;
		try {
			const result = await form.validateForm();
			if (result.valid) {
				const member = await invoke('update_member', {
					payload: result.data
				});
				toast.success('Data saved successfully!');
				fetchMember();
			} else {
				toast.error('Data is not valid!');
			}
		} catch (error) {
			console.log(error);
			const errorMessage = (error as ErrorResponse)?.message || 'Failed to edit member!';
			toast.error(errorMessage);

			return;
		} finally {
			isLoading = false;
		}
	}

	async function handleCancel() {
		await goto('/members');
	}

	let date_of_birth = $state<DateValue | undefined>();

	$effect(() => {
		date_of_birth = $formData.date_of_birth ? parseDate($formData.date_of_birth) : undefined;
	});

	function handleDobChange(newValue: DateValue | undefined) {
		$formData.date_of_birth = newValue ? newValue.toString() : null;
	}

	onMount(async () => {
		setHeader({
			title: 'Edit Member',
			showBackButton: true
		});
		if (memberId) {
			fetchMember();
		}
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">Member</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-10">
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
								value={date_of_birth}
								onValueChange={handleDobChange}
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
						<Button variant="outline" on:click={handleCancel} class="w-full">Cancel</Button>
						<Form.Button type="submit" class="w-full">Save</Form.Button>
					</div>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
