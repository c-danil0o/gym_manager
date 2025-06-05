<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import type { z } from 'zod';
	import { newMemberSchema, type NewMemberTypeSchema } from '$lib/schemas/new_member_schema';
	import type { Member } from '$lib/models/member';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import DateField from '$lib/components/date-field/date-field.svelte';

	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';
	import { setHeader, setLoading } from '$lib/stores/state';
	import { onMount } from 'svelte';
	import type { ErrorResponse } from '$lib/models/error';
	import { m } from '$lib/paraglide/messages';

	let newMember: null | Member = null;
	let showMembershipPrompt = false;

	const initialValues: z.infer<NewMemberTypeSchema> = {
		card_id: '',
		first_name: '',
		last_name: '',
		email: '',
		phone: '',
		date_of_birth: null
	};

	const form = superForm(initialValues, {
		validators: zodClient(newMemberSchema),
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

	let placeholder: DateValue = today(getLocalTimeZone());

	function handleDateChange(newValue: DateValue | undefined) {
		$formData.date_of_birth = newValue ? newValue.toString() : null;
	}

	async function handleSubmit() {
		setLoading(true);
		newMember = null;
		try {
			const result = await form.validateForm();
			if (result.valid) {
				const member: Member = await invoke('add_member', { payload: result.data });
				newMember = member;
				showMembershipPrompt = true;
				toast.success(m.new_member_add_success());
			} else {
				toast.error(m.toast_error_invalid_data());
			}
		} catch (error) {
			showMembershipPrompt = false;
			// TRANSLATE ERROR
			const errorMessage = (error as ErrorResponse)?.message || 'Failed to add new member!';
			toast.error(errorMessage);
			return;
		} finally {
			setLoading(false);
		}
	}
	async function handleCancel() {
		await goto('/members');
	}

	async function handleCancelPrompt() {
		if (newMember) {
			await goto(`/members/${newMember.id}`);
		} else {
			await goto('/members');
		}
	}

	async function assignMembership() {
		if (newMember) {
			await goto(`/members/${newMember.id}/new-membership`);
		}
		showMembershipPrompt = false;
	}
	onMount(() => {
		setHeader({
			title: 'New Member',
			showBackButton: true
		});
	});
</script>

<div class="container mx-auto p-4 md:p-8 max-w-2xl">
	<Card.Root>
		<Card.Header>
			<Card.Title class="text-2xl">{m.add_new_member()}</Card.Title>
		</Card.Header>
		<Card.Content>
			<form use:enhance method="post" onsubmit={handleSubmit} class="space-y-6">
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
							<DateField {...props} {placeholder} onValueChange={handleDateChange} locale="bs-BA" />
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
					<Form.Button variant="outline" onclick={handleCancel} class="w-full"
						>{m['common.cancel']()}</Form.Button
					>
					<Form.Button type="submit" class="w-full">{m['common.confirm']()}</Form.Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
	{#if newMember}
		<AlertDialog.Root bind:open={showMembershipPrompt}>
			<AlertDialog.Content>
				<AlertDialog.Header>
					<AlertDialog.Title>{m.membership_prompt()}</AlertDialog.Title>
					<AlertDialog.Description>
						{m['common.member']()}
						<b
							>{newMember.first_name}
							{newMember.last_name}</b
						>
						{m.membership_prompt_desc()}
					</AlertDialog.Description>
				</AlertDialog.Header>
				<AlertDialog.Footer>
					<AlertDialog.Cancel onclick={handleCancelPrompt}
						>{m.membership_prompt_cancel()}</AlertDialog.Cancel
					>
					<AlertDialog.Action onclick={assignMembership}
						>{m.membership_prompt_yes()}</AlertDialog.Action
					>
				</AlertDialog.Footer>
			</AlertDialog.Content>
		</AlertDialog.Root>
	{/if}
</div>
