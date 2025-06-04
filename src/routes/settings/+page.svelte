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
	import { setHeader, setLoading } from '$lib/stores/state';
	import { onMount } from 'svelte';
	import type { ErrorResponse } from '$lib/models/error';
	import * as Select from '$lib/components/ui/select';
	import Separator from '$lib/components/ui/separator/separator.svelte';

	const languages = [
		{ id: 'en', name: 'English' },
		{ id: 'rs', name: 'Srpski' }
	];

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

	async function handleSubmit() {
		setLoading(true);
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
			console.log(error);
			const errorMessage =
				(error as ErrorResponse)?.message || 'Failed to add new membership type!';
			toast.error(errorMessage);
			return;
		} finally {
			setLoading(false);
		}
	}
	async function handleCancel() {
		await goto('/memberships');
	}
	onMount(() => {
		setHeader({
			title: 'Settings'
		});
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Content>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit} class="space-y-6">
			<Card.Title class="text-xl">Locale</Card.Title>
				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">Language</Form.Label>

							<Select.Root type="single" bind:value={$formData.name}>
								<Select.Trigger {...props}>
									{formData?.name ? $formData.name : 'Select language'}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each languages as type (type.id)}
											<Select.Item value={String(type.id)} label={type.name}
												>{type.name}</Select.Item
											>
										{/each}
										{#if languages.length === 0}
											<div class="px-2 py-1.5 text-sm text-muted-foreground">
												No languages available.
											</div>
										{/if}
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">Timezone</Form.Label>

							<Select.Root type="single" bind:value={$formData.name}>
								<Select.Trigger {...props}>
									{formData?.name ? $formData.name : 'Select timezone'}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each languages as type (type.id)}
											<Select.Item value={String(type.id)} label={type.name}
												>{type.name}</Select.Item
											>
										{/each}
										{#if languages.length === 0}
											<div class="px-2 py-1.5 text-sm text-muted-foreground">
												No languages available.
											</div>
										{/if}
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Separator />

				<Card.Title class="text-xl">Appearance</Card.Title>
				<Form.Field {form} name="duration_days">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">Theme</Form.Label>
							<Select.Root type="single" bind:value={$formData.name}>
								<Select.Trigger {...props}>
									{formData?.name ? $formData.name : 'Select theme'}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
											<Select.Item value='light' label='Light' />
											<Select.Item value='dark' label='Dark' />
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>
				<Separator />

				<Card.Title class="text-xl">Backup</Card.Title>
				<Form.Field {form} name="duration_days">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">Backup URL</Form.Label>
							<Input {...props} type="url" bind:value={$formData.duration_days} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="visit_limit">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">Backup period</Form.Label>
							<Select.Root type="single" bind:value={$formData.name}>
								<Select.Trigger {...props}>
									{formData?.name ? $formData.name : 'Select period'}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each languages as type (type.id)}
											<Select.Item value={String(type.id)} label={type.name}
												>{type.name}</Select.Item
											>
										{/each}
										{#if languages.length === 0}
											<div class="px-2 py-1.5 text-sm text-muted-foreground">
												No languages available.
											</div>
										{/if}
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<div class="flex gap-20 justify-around mt-10">
					<Button variant="outline" onclick={handleCancel} class="w-full">Cancel</Button>
					<Form.Button type="submit" class="w-full">Save</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
