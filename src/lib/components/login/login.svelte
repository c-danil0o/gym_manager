<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import gym from '../../../images/gym_picture.jpg';

	import { auth } from '$lib/stores/auth'; // Use $lib alias for SvelteKit
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { toast } from 'svelte-sonner';
	import { m } from '$lib/paraglide/messages';
	import { setLoading } from '$lib/stores/state';

	const formSchema = z.object({
		username: z.string().min(2, { message: '' }).max(20, { message: '' }),
		password: z.string().min(5, { message: '' }).max(20, { message: '' })
	});

	const defaultValues = {
		username: '',
		password: ''
	};

	const form = superForm(defaultValues, {
		validators: zodClient(formSchema),
		dataType: 'json',
		SPA: true,
		taintedMessage: null
	});
	const { form: formData, enhance } = form;

	async function handleSubmit() {
		setLoading(true);
		const result = await form.validateForm();
		if (result.valid) {
			auth.clearError();
			const loginSuccess = await auth.login(result.data.username, result.data.password);
			if (loginSuccess) {
				toast.success(m['login.toast_success']());
			} else {
				toast.error(m['login.toast_fail']());
			}
		} else {
			toast.error(m['login.toast_validation']());
		}
		setLoading(false);
	}
</script>

<div class="w-full lg:grid lg:min-h-[600px] h-screen lg:grid-cols-2 xl:min-h-[800px]">
	<div class="flex items-center justify-center py-12 w-full h-full">
		<Card.Root class="w-full max-w-sm">
			<Card.Header>
				<Card.Title class="text-2xl">{m['login.title']()}</Card.Title>
				<Card.Description>{m['login.description']()}</Card.Description>
			</Card.Header>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit}>
				<Card.Content class="grid gap-4">
					<Form.Field {form} name="username">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="font-semibold">{m['login.username']()}</Form.Label>
								<Input {...props} type="text" bind:value={$formData.username} />
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

					<Form.Button type="submit" class="w-full">{m['login.title']()}</Form.Button>
				</Card.Content>
			</form>
		</Card.Root>
	</div>
	<div class="bg-muted hidden lg:block">
		<img
			src={gym}
			alt="placeholder"
			width="1920"
			height="1080"
			class="h-full w-full object-cover dark:brightness-[0.8] grayscale"
		/>
	</div>
</div>
