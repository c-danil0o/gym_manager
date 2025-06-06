<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Select from '$lib/components/ui/select';
	import { setHeader, setLoading } from '$lib/stores/state';
	import { onMount } from 'svelte';
	import type { ErrorResponse } from '$lib/models/error';
	import { m } from '$lib/paraglide/messages';
	import { translateErrorCode } from '$lib/utils';
	import { newUserSchema, type NewUserSchemaType } from '$lib/schemas/user_schema';
	import { requireRole } from '../../guards';

	const initialValues: z.infer<NewUserSchemaType> = {
		username: '',
		role: '',
		password: ''
	};
	const roles = [
		{ value: 'user', label: m.user() },
		{ value: 'admin', label: m.admin() }
	];

	const form = superForm(initialValues, {
		validators: zodClient(newUserSchema),
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
				await invoke('save_user', { payload: result.data });
				toast.success(m.new_user_add_success());
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
				toast.error(m.toast_error_new_user_fail());
			}
		} finally {
			setLoading(false);
		}
	}
	async function handleCancel() {
		await goto('/users');
	}
	onMount(() => {
		requireRole('admin');
		setHeader({
			title: m.add_new_user(),
			showBackButton: true
		});
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">{m.add_new_user()}</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit} class="space-y-6">
				<Form.Field {form} name="username">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.username()}</Form.Label>
							<Input {...props} type="text" bind:value={$formData.username} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="role">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m['common.duration']()}</Form.Label>
							<Select.Root type="single" bind:value={$formData.role}>
								<Select.Trigger {...props}>
									{$formData?.role
										? roles.find((role) => role.value === $formData.role)?.label
										: m.select_role()}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each roles as type (type.value)}
											<Select.Item value={type.value} label={type.label}>{type.label}</Select.Item>
										{/each}
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="password">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m['login.password']()}</Form.Label>
							<Input {...props} type="password" bind:value={$formData.password} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<div class="flex gap-20 justify-around">
					<Button variant="outline" onclick={handleCancel} class="w-full"
						>{m['common.cancel']()}</Button
					>
					<Form.Button type="submit" class="w-full">{m['common.save']()}</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
