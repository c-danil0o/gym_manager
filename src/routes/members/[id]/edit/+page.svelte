<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import { parseDate, today, type DateValue, getLocalTimeZone } from '@internationalized/date';
	import { onMount } from 'svelte';
	import { editMemberSchema, type EditMemberTypeSchema } from '$lib/schemas/edit_member_schema';
	import type { Member } from '$lib/models/member';
	import { setHeader, setLoading } from '$lib/stores/state';
	import Button from '$lib/components/ui/button/button.svelte';
	import type { ErrorResponse } from '$lib/models/error';
	import { m } from '$lib/paraglide/messages';
	import { translateErrorCode } from '$lib/utils';
	import DatePicker from '$lib/components/date-picker/date-picker.svelte';

	let error: string | null = $state(null);
	const memberId = $derived(page.params.id);
	const locale = m.locale_code() || 'bs-BA';


	async function fetchMember() {
		setLoading(true);
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
			toast.error(m.failed_to_load_member());
		} finally {
			setLoading(false);
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
		setLoading(true);
		try {
			const result = await form.validateForm();
			if (result.valid) {
				const member = await invoke('update_member', {
					payload: result.data
				});
				toast.success(m.member_update_success());
				fetchMember();
			} else {
				toast.error(m['toast_error_invalid_data']());
			}
		} catch (error) {
			console.log(error);
			const errorMessage = error as ErrorResponse;
			if (errorMessage?.error_code && errorMessage?.params) {
				toast.error(translateErrorCode(errorMessage.error_code, errorMessage.params));
			} else {
				toast.error(m.toast_error_update_member_fail());
			}

			return;
		} finally {
			setLoading(false);
		}
	}

	async function handleCancel() {
		await goto('/members');
	}

	let date_of_birth = $state<DateValue | undefined>();
	const todayDate = today(getLocalTimeZone());

	$effect(() => {
		date_of_birth = $formData.date_of_birth ? parseDate($formData.date_of_birth) : undefined;
	});

	function handleDobChange(newValue: DateValue | undefined) {
		$formData.date_of_birth = newValue ? newValue.toString() : null;
	}

	onMount(async () => {
		setHeader({
			title: m.edit_member(),
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
			<Card.Title class="text-2xl">{m['common.member']()}</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-10">
				<div class="space-y-6">
					<Form.Field {form} name="first_name">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.first_name()}</Form.Label>
								<Input {...props} type="text" bind:value={$formData.first_name} />
								<Form.FieldErrors />
							{/snippet}
						</Form.Control>
					</Form.Field>

					<Form.Field {form} name="last_name">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.last_name()}</Form.Label>
								<Input {...props} type="text" bind:value={$formData.last_name} />
								<Form.FieldErrors />
							{/snippet}
						</Form.Control>
					</Form.Field>

					<Form.Field {form} name="email">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.email()}</Form.Label>
								<Input {...props} type="email" bind:value={$formData.email} />
								<Form.Description class="text-xs">{m.optional()}</Form.Description>
								<Form.FieldErrors />
							{/snippet}
						</Form.Control>
					</Form.Field>

					<Form.Field {form} name="phone">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.phone()}</Form.Label>
								<Input {...props} type="text" bind:value={$formData.phone} />
								<Form.Description class="text-xs">{m.optional()}</Form.Description>
								<Form.FieldErrors />
							{/snippet}
						</Form.Control>
					</Form.Field>

					<Form.Field {form} name="date_of_birth">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.date_of_birth()}</Form.Label>
								<DatePicker
									{...props}
									value={date_of_birth}
									onValueChange={handleDobChange}
									locale={locale}
									maxValue={todayDate}
								/>
								<Form.FieldErrors />
								<Form.Description class="text-xs">{m.optional()}</Form.Description>
							{/snippet}
						</Form.Control>
					</Form.Field>

					<Form.Field {form} name="card_id">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m.card_number()}</Form.Label>
								<Input {...props} type="text" bind:value={$formData.card_id} />
								<Form.Description class="text-xs">{m.use_scanner_or_enter()}</Form.Description>
								<Form.FieldErrors />
							{/snippet}
						</Form.Control>
					</Form.Field>

					<div class="flex gap-20 justify-around">
						<Button variant="outline" onclick={handleCancel} class="w-full">{m['common.cancel']()}</Button>
						<Form.Button type="submit" class="w-full">{m['common.save']()}</Form.Button>
					</div>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
