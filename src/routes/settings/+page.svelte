<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
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
	import type { BackupMetadata } from '$lib/models/backup_metadata';
	import { translateErrorCode } from '$lib/utils';
	import { DateFormatter } from '@internationalized/date';
	import Label from '$lib/components/ui/label/label.svelte';
	import Switch from '$lib/components/ui/switch/switch.svelte';
	import { relaunch } from '@tauri-apps/plugin-process';

	const languages = [
		{ id: 'en', name: 'English' },
		{ id: 'rs', name: 'Srpski' }
	];

	const locale = m.locale_code() || 'bs-BA';
	let isBackupDialogOpen = $state(false);
	let isRestoreDialogOpen = $state(false);

	const df = new DateFormatter(locale, {
		dateStyle: 'short',
		timeStyle: 'short'
	});

	const initialValues: z.infer<SettingsSchemaType> = {
		gym_name: 'Gym',
		backup_enabled: false,
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
	let backups = $state<BackupMetadata[] | []>([]);
	let selectedBackup = $state<string | undefined>();

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

	async function loadBackups() {
		try {
			if (!$formData.backup_url) return;
			const backupsData = await invoke<BackupMetadata[]>('get_remote_backup_metadata');
			if (backups) {
				backups = backupsData.slice(0, 5);
				for (const backup of backups) {
					backup.label = df.format(new Date(backup.lastModified));
				}
			} else {
				backups = [];
			}
		} catch (error) {
			console.error('Failed to get backups:', error);
			const errorMessage = error as ErrorResponse;
			if (errorMessage?.error_code && errorMessage?.params) {
				toast.error(translateErrorCode(errorMessage.error_code, errorMessage.params));
			} else {
				toast.error(m.failed_to_load_backups());
			}
		}
	}

	async function handleSubmit() {
		setLoading(true);
		try {
			const result = await form.validateForm();
			if (result.valid) {
				backups = [];
				await invoke('update_app_settings', { payload: result.data });
				toast.success(m.settings_updated());
				loadBackups();
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

	async function triggerBackup() {
		isBackupDialogOpen = false;
		setLoading(true);
		try {
			await invoke('trigger_backup');
			setLoading(false);
			loadBackups();
			toast.success(m.backup_success());
		} catch (error: any) {
			console.log(error);
			const errorMessage = error as ErrorResponse;
			if (errorMessage?.error_code && errorMessage?.params) {
				toast.error(translateErrorCode(errorMessage.error_code, errorMessage.params));
			} else {
				toast.error(m.failed_to_trigger_backup());
			}
			setLoading(false);
		}
	}

	async function triggerRestore() {
		isRestoreDialogOpen = false;
		setLoading(true);
		console.log(selectedBackup);
		if (!selectedBackup || selectedBackup === '') {
			toast.error(m.please_select_backup());
			setLoading(false);
			return;
		}
		try {
			await invoke('restore_from_backup', { versionId: selectedBackup });
			setLoading(false);
			toast.warning(m.restore_success());
			setTimeout(async () => {
				await relaunch();
			}, 10000);
		} catch (error: any) {
			console.log(error);
			const errorMessage = error as ErrorResponse;
			if (errorMessage?.error_code && errorMessage?.params) {
				toast.error(translateErrorCode(errorMessage.error_code, errorMessage.params));
			} else {
				toast.error(m.failed_to_restore_backup());
			}
			setLoading(false);
		}
	}
	onMount(async () => {
		requireRole('admin');
		setHeader({
			title: m['common.settings']()
		});
		setLoading(true);
		await loadSettings();
		await loadBackups();
		setLoading(false);
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Content>
			<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-6">
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
				<Form.Field {form} name="gym_name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="font-semibold">{m.gym_name()}</Form.Label>
							<Input {...props} type="text" bind:value={$formData.gym_name} />
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>
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

				<Form.Field {form} name="backup_enabled">
					<Form.Control>
						{#snippet children({ props })}
							<div class="space-x-3 flex items-center">
								<Form.Label class="font-semibold">{m.enable_backup()}</Form.Label>
								<Switch {...props} bind:checked={$formData.backup_enabled} />
							</div>
							<Form.FieldErrors />
						{/snippet}
					</Form.Control>
				</Form.Field>
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

				<div class="w-full space-y-2">
					<Label class="font-semibold">{m.last_successfull_backup()}</Label>
					<Input
						type="text"
						readonly
						value={backups ? backups[0]?.label || m.no_backup_found() : m.no_backup_found()}
					/>

					<div class="flex flex-col md:flex-row gap-4 w-full justify-between items-center pt-2">
						<div class="w-1/2 space-y-2">
							<Label class="font-semibold">{m.backups()}</Label>
							<Select.Root type="single" bind:value={selectedBackup}>
								<Select.Trigger>
									{selectedBackup && backups
										? backups.find((b) => b.versionId === selectedBackup)?.label
										: m.select_backup()}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each backups as type (type.versionId)}
											<Select.Item value={type.versionId} label={type.label}
												>{type.label}</Select.Item
											>
										{/each}
									</Select.Group>
								</Select.Content>
							</Select.Root>
						</div>
						<div class="w-1/2 space-y-2">
							<AlertDialog.Root bind:open={isBackupDialogOpen}>
								<AlertDialog.Trigger class="w-full" type="button">
									<Button class="w-full" variant="secondary">{m.trigger_backup()}</Button>
								</AlertDialog.Trigger>
								<AlertDialog.Content>
									<AlertDialog.Header>
										<AlertDialog.Title>{m['common.are_you_sure']()}</AlertDialog.Title>
										<AlertDialog.Description>
											{m.trigger_backup_desc()}</AlertDialog.Description
										>
									</AlertDialog.Header>
									<AlertDialog.Footer>
										<AlertDialog.Cancel>{m.cancel()}</AlertDialog.Cancel>
										<AlertDialog.Action
											onclick={() => {
												triggerBackup();
											}}>{m.confirm()}</AlertDialog.Action
										>
									</AlertDialog.Footer>
								</AlertDialog.Content>
							</AlertDialog.Root>
							<AlertDialog.Root bind:open={isRestoreDialogOpen}>
								<AlertDialog.Trigger class="w-full" type="button">
									<Button class="w-full" variant="destructive">{m.restore_backup()}</Button>
								</AlertDialog.Trigger>
								<AlertDialog.Content>
									<AlertDialog.Header>
										<AlertDialog.Title>{m['common.are_you_sure']()}</AlertDialog.Title>
										<AlertDialog.Description>
											{m.restore_backup_desc()}</AlertDialog.Description
										>
									</AlertDialog.Header>
									<AlertDialog.Footer>
										<AlertDialog.Cancel>{m.cancel()}</AlertDialog.Cancel>
										<AlertDialog.Action
											onclick={() => {
												triggerRestore();
											}}>{m.confirm()}</AlertDialog.Action
										>
									</AlertDialog.Footer>
								</AlertDialog.Content>
							</AlertDialog.Root>
						</div>
					</div>

					<div class="flex gap-20 justify-around mt-10">
						<Button variant="outline" onclick={handleCancel} class="w-full"
							>{m['common.cancel']()}</Button
						>
						<Form.Button type="submit" class="w-full">{m['common.save']()}</Form.Button>
					</div>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
