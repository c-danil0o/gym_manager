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
	import { setHeader, setLoading } from '$lib/stores/state';
	import { onMount } from 'svelte';
	import type { ErrorResponse } from '$lib/models/error';
	import * as Select from '$lib/components/ui/select';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import { type SettingsSchemaType, settingsSchema } from '$lib/schemas/settings_schema';
	import { m } from '$lib/paraglide/messages';
	import { requireRole } from '../guards';

	const languages = [
		{ id: 'en', name: 'English' },
		{ id: 'rs', name: 'Srpski' }
	];

	const initialValues: z.infer<SettingsSchemaType> = {
		language: 'en',
		timezone: 'UTC',
		theme: 'light',
		backup_url: '',
		backup_period_hours: 0
	};

	const form = superForm(initialValues, {
		validators: zodClient(settingsSchema),
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

	async function loadSettings() {
		try {
			const settings = await invoke<z.infer<SettingsSchemaType>>('get_app_settings');
			formData.set(settings);
		} catch (error) {
			console.error('Failed to load settings:', error);
			toast.error(m['main.toast_failed_settings']());
		}
	}

	async function handleSubmit() {
		setLoading(true);
		try {
			const result = await form.validateForm();
			if (result.valid) {
				await invoke('update_app_settings', { payload: result.data });
				toast.success(m.settings_updated());
			} else {
				toast.error('Data is not valid!');
			}
		} catch (error) {
			console.log(error);
			const errorMessage = (error as ErrorResponse)?.message || m.settings_update_failed();
			toast.error(errorMessage);
			return;
		} finally {
			setLoading(false);
		}
	}
	async function handleCancel() {
		await goto('/');
	}
	onMount(async () => {
		requireRole('admin');
		setHeader({
			title: 'Settings'
		});
		setLoading(true);
		await loadSettings();
		setLoading(false);
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Content>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit} class="space-y-6">
				<Card.Title class="text-xl">{m.locale()}</Card.Title>
				<Form.Field {form} name="language">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.language()}</Form.Label>

							<Select.Root type="single" bind:value={$formData.language}>
								<Select.Trigger {...props}>
									{languages.find((l) => l.id === $formData.language)
										? languages.find((l) => l.id === $formData.language)?.name
										: m.select_language()}
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
												{m.no_languages()}.
											</div>
										{/if}
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="timezone">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.timezone()}</Form.Label>
							<Input {...props} type="text" bind:value={$formData.timezone} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Separator />

				<Card.Title class="text-xl">{m.appearance()}</Card.Title>
				<Form.Field {form} name="theme">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.theme()}</Form.Label>
							<Select.Root type="single" bind:value={$formData.theme}>
								<Select.Trigger {...props}>
									{$formData?.theme ? $formData.theme : m.select_theme()}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										<Select.Item value="light" label="Light" />
										<Select.Item value="dark" label="Dark" />
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>
				<Separator />

				<Card.Title class="text-xl">{m.backup()}</Card.Title>
				<Form.Field {form} name="backup_url">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.backup_url()}</Form.Label>
							<Input {...props} type="text" bind:value={$formData.backup_url} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<Form.Field {form} name="backup_period_hours">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.backup_period()}</Form.Label>
							<Select.Root
								type="single"
								value={String($formData.backup_period_hours)}
								onValueChange={(value) => {
									$formData.backup_period_hours = value ? parseInt(value) : undefined;
								}}
							>
								<Select.Trigger {...props}>
									{$formData?.backup_period_hours
										? $formData.backup_period_hours + 'h'
										: m.select_period()}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										<Select.Item value="3" label="3h" />
										<Select.Item value="6" label="6h" />
										<Select.Item value="12" label="12h" />
										<Select.Item value="24" label="24h" />
									</Select.Group>
								</Select.Content>
							</Select.Root>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>

				<div class="flex gap-20 justify-around mt-10">
					<Button variant="outline" onclick={handleCancel} class="w-full"
						>{m['common.cancel']()}</Button
					>
					<Form.Button type="submit" class="w-full">{m['common.save']()}</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
