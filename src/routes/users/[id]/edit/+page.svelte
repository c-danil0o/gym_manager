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
	import {
		changePasswordSchema,
		updateUserSchema,
		type ChangePasswordSchemaType,
		type UpdateUserSchemaType
	} from '$lib/schemas/user_schema';
	import type { User } from '$lib/models/user';
	import { page } from '$app/state';
	import Separator from '$lib/components/ui/separator/separator.svelte';

	const userId = $derived(page.params.id);
	const initialValues: z.infer<UpdateUserSchemaType> = {
		username: '',
		role: ''
	};

	const passInitialValues: z.infer<ChangePasswordSchemaType> = {
		new_password: '',
		user_id: 0
	};
	const roles = [
		{ value: 'user', label: m.user() },
		{ value: 'admin', label: m.admin() }
	];

	const form = superForm(initialValues, {
		validators: zodClient(updateUserSchema),
		syncFlashMessage: true,
		dataType: 'json',
		SPA: true,
		taintedMessage: null,
		resetForm: false,
		onUpdated({ form: currentForm }) {
			if (!currentForm.valid) console.log('Client errors:', currentForm.errors);
		}
	});

	const form2 = superForm(passInitialValues, {
		validators: zodClient(changePasswordSchema),
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
	const { form: formData2, enhance: enhance2 } = form2;

	async function fetchUser() {
		try {
			const user = await invoke<User>('get_user_by_id', { userId: Number(userId) });
			if (user) {
				$formData.username = user.username;
				$formData.role = user.role;
				$formData.id = user.id;
				$formData2.user_id = user.id;
			} else {
				toast.error(m.no_user_found());
			}
		} catch (error) {
			console.error(error);
			toast.error(m.toast_failed_fetch_users());
		}
	}

	async function handleSubmit() {
		setLoading(true);
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('save_user', { payload: result.data });
				toast.success(m.user_update_success());
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
				toast.error(m.toast_error_update_user_fail());
			}
		} finally {
			setLoading(false);
		}
	}

	async function changePassword() {
		setLoading(true);
		try {
			const result = await form2.validateForm();
			if (result.valid) {
				await invoke('change_user_password', {
					userId: result.data.user_id,
					newPassword: result.data.new_password
				});
				toast.success(m.password_change_success());
				handleCancel();
			} else {
				toast.error(m.toast_error_invalid_data());
			}
		} catch (error) {
			console.log(error);
			toast.error(m.password_change_fail());
		} finally {
			setLoading(false);
		}
	}

	async function handleCancel() {
		await goto('/users');
	}
	onMount(() => {
		setLoading(true);
		setHeader({
			title: m.update_user(),
			showBackButton: true
		});
		fetchUser();
		setLoading(false);
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">{m.update_user()}</Card.Title>
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

				<div class="flex gap-20 justify-around">
					<Form.Button type="submit" class="w-full">{m['common.save']()}</Form.Button>
				</div>
			</form>

			<Separator class="my-7" />

			<Card.Title class="text-2xl mb-7">{m.change_password()}</Card.Title>

			<form use:enhance2 method="post" on:submit|preventDefault={changePassword} class="space-y-6">
				<Form.Field form={form2} name="new_password">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.new_password()}</Form.Label>
							<Input {...props} type="password" bind:value={$formData2.new_password} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Button class="w-full">{m.change()}</Form.Button>
			</form>
		</Card.Content>
	</Card.Root>
</div>
