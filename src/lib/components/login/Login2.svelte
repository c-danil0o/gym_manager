<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import gym from '../../../images/gym_picture.jpg';

	import { auth } from '$lib/stores/auth'; // Use $lib alias for SvelteKit
	import { type SuperValidated, type Infer, superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { toast } from 'svelte-sonner';

	let loading = false;

	export const formSchema = z.object({
		username: z
			.string()
			.min(2, { message: 'Username must be between 2 and 50 characters' })
			.max(20, { message: 'Username must be between 2 and 50 characters' }),
		password: z
			.string()
			.min(5, { message: 'Password must be between 5 and 20 characters' })
			.max(20, { message: 'Password must be between 5 and 20 characters' })
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
		loading = true;
		const result = await form.validateForm();
		if (result.valid) {
			auth.clearError();
			const loginSuccess = await auth.login(result.data.username, result.data.password);
			if (loginSuccess) {
				toast.success('Successfully logged in!');
			} else {
				toast.error('Login failed!');
			}
		} else {
			toast.error('Data is not valid!');
		}
		loading = false;
	}
</script>

<div class="w-full lg:grid lg:min-h-[600px] h-screen lg:grid-cols-2 xl:min-h-[800px]">
	<div class="flex items-center justify-center py-12 w-full h-full">
		<Card.Root class="w-full max-w-sm">
			<Card.Header>
				<Card.Title class="text-2xl">Login</Card.Title>
				<Card.Description>Access to gym management system.</Card.Description>
			</Card.Header>
			<form use:enhance method="post" on:submit|preventDefault={handleSubmit}>
				<Card.Content class="grid gap-4">
					<Form.Field {form} name="username">
						<Form.Control let:attrs>
							<Form.Label class="font-semibold">Username</Form.Label>
							<Input {...attrs} type="text" bind:value={$formData.username} />
							<Form.FieldErrors />
						</Form.Control>
					</Form.Field>

					<Form.Field {form} name="password">
						<Form.Control let:attrs>
							<Form.Label class="font-semibold">Password</Form.Label>
							<Input {...attrs} type="password" bind:value={$formData.password} />
							<Form.FieldErrors />
						</Form.Control>
					</Form.Field>

					<Form.Button type="submit" class="w-full">Login</Form.Button>
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
			class="h-full w-full object-cover dark:brightness-[0.2] dark:grayscale"
		/>
	</div>
</div>
