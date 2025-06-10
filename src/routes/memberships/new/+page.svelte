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
	import { m } from '$lib/paraglide/messages';
	import { translateErrorCode } from '$lib/utils';
	import { requireRole } from '../../guards';
	import Checkbox from '$lib/components/ui/checkbox/checkbox.svelte';

	const initialValues: z.infer<MembershipTypeSchema> = {
		name: '',
		duration_days: null,
		visit_limit: 0,
		enter_by: null,
		price: 0,
		description: '',
		is_active: true
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
				toast.success(m.toast_new_membership_type_success());
				handleCancel();
			} else {
				toast.error(m.toast_error_invalid_data());
			}
		} catch (error) {
			console.log(error);
			const errorMessage = error as ErrorResponse;
			if (errorMessage?.error_code && errorMessage?.params) {
				toast.error(translateErrorCode(errorMessage.error_code, errorMessage.params));
			} else {
				toast.error(m.toast_error_membership_type_fail());
			}
		} finally {
			setLoading(false);
		}
	}
	async function handleCancel() {
		await goto('/memberships');
	}
	onMount(() => {
		requireRole('admin');
		setHeader({
			title: m.new_membership_type(),
			showBackButton: true
		});
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">{m.add_new_memberhsip_type()}</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit} class="space-y-6">
				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m['common.name']()}</Form.Label>
							<Input {...props} type="text" bind:value={$formData.name} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="duration_days">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m['common.duration']()}</Form.Label>
							<div class="relative flex">
								<Input {...props} type="text" bind:value={$formData.duration_days} class="pr-15" />
								<span
									class="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground pointer-events-none text-xs"
								>
									{m['common.days']()}
								</span>
							</div>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="visit_limit">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m['common.visit_limit']()}</Form.Label>
							<Input {...props} type="text" bind:value={$formData.visit_limit} />
							<Form.Description class="text-sm text-muted-foreground">
								{m.visit_limit_desc()}
							</Form.Description>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="enter_by">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.enter_by_with_desc()}</Form.Label>
							<div class="relative flex">
								<Input
									{...props}
									type="number"
									min="1"
									max="24"
									placeholder={m.optional()}
									bind:value={$formData.enter_by}
									class="pr-15"
								/>
								<span
									class="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground pointer-events-none text-xs"
								>
									h
								</span>
							</div>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="price">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m['common.price']()}</Form.Label>
							<div class="relative flex">
								<Input {...props} type="number" class="pr-15" bind:value={$formData.price} />
								<span
									class="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground pointer-events-none text-xs"
								>
									{m.locale_currency()}
								</span>
							</div>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="description">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m['common.description']()}</Form.Label>
							<Textarea {...props} bind:value={$formData.description} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="is_active">
					<Form.Control>
						{#snippet children({ props })}
							<div class="space-x-3 flex items-center">
								<Form.Label class="font-semibold">{m.is_active()}</Form.Label>
								<Checkbox {...props} bind:checked={$formData.is_active} />
							</div>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<div class="flex gap-20 justify-around">
					<Button variant="outline" onclick={handleCancel} class="w-full"
						>{m['common.cancel']()}</Button
					>
					<Form.Button type="submit" class="w-full">{m['common.confirm']()}</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
