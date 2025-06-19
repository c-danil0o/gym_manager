<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	// Shadcn UI imports
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import PlusCircle from 'lucide-svelte/icons/plus-circle';
	import Trash2 from 'lucide-svelte/icons/trash-2';
	import Pencil from 'lucide-svelte/icons/pencil';
	import { setHeader, setLoading } from '$lib/stores/state';
	import Input from '$lib/components/ui/input/input.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { User } from '$lib/models/user';
	import { translateRole } from '$lib/utils';
	import { requireRole } from '../guards';

	let users: User[] = [];
	let filteredUsers: User[] = [];
	let error: string | null = null;

	async function fetchUsers() {
		error = null;
		try {
			const result = await invoke<User[]>('get_all_users');
			users = result || [];
			filteredUsers = users;
		} catch (e: any) {
			error = e.message || m.toast_failed_fetch_users();
			console.error('Error fetching users', e);
			toast.error(m.toast_failed_fetch_users());
		}
	}

	onMount(async () => {
		requireRole('admin');
		setHeader({
			title: m['common.users'](),
			showBackButton: false
		});
		setLoading(true);
		await fetchUsers();
		setLoading(false);
	});

	function handleAddNew() {
		goto('/users/new');
	}

	async function handleDelete(userId: number) {
		try {
			await invoke('delete_user', { userId: userId });
			toast.success(m.user_delete_success());
			fetchUsers();
		} catch (e: any) {
			console.error('Error deleting user:', e);
			toast.error(m.user_delete_fail());
		}
	}

	function handleEdit(userId: number) {
		goto(`/users/${userId}/edit`);
	}

	function onSearchChange(value: string) {
		if (value.trim() === '') {
			filteredUsers = users;
		} else {
			const lowerValue = value.toLowerCase();
			filteredUsers = users.filter((user) => user.username.toLowerCase().includes(lowerValue));
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<Input
			placeholder={m['common.search']() + '...'}
			oninput={(e) => {
				if (onSearchChange) {
					onSearchChange(e.currentTarget.value);
				}
			}}
			class="h-8 w-[150px] lg:w-[250px] bg-card"
		/>
		<Button onclick={handleAddNew} class="h-8 text-xs">
			<PlusCircle class="mr-2 h-4 w-4" />
			{m['common.add']()}
		</Button>
	</div>

	{#if error}
		<Card.Root class="border-destructive">
			<Card.Header>
				<Card.Title class="text-destructive">{m['common.error']()}</Card.Title>
			</Card.Header>
			<Card.Content>
				<p>{error}</p>
				<Button onclick={fetchUsers} variant="outline" class="mt-4"
					>{m['common.try_again']()}</Button
				>
			</Card.Content>
		</Card.Root>
	{:else if users.length === 0}
		<Card.Root>
			<Card.Content class="pt-6">
				<p class="text-center text-muted-foreground">{m.no_users_found()}</p>
				<p class="text-center mt-2">
					<Button onclick={handleAddNew} variant="link">{m.add_new_user()}</Button>
				</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<Card.Root>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>ID</Table.Head>
						<Table.Head>{m.username()}</Table.Head>
						<Table.Head>{m.role()}</Table.Head>
						<Table.Head class="text-right pr-12">{m['common.actions']()}</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredUsers as type (type.id)}
						<Table.Row>
							<Table.Cell class="font-medium">{type.id}</Table.Cell>
							<Table.Cell class="font-medium">{type.username}</Table.Cell>
							<Table.Cell class="font-medium">{translateRole(type.role)}</Table.Cell>
							<Table.Cell class="text-right pr-8 space-x-2">
								<Button
									onclick={() => handleEdit(type.id)}
									variant="outline"
									size="icon"
									title={m['common.edit']()}
								>
									<Pencil class="h-4 w-4" />
								</Button>
								<AlertDialog.Root>
									<AlertDialog.Trigger>
										<Button variant="destructive" size="icon" title={m['common.delete']()}>
											<Trash2 class="h-4 w-4" />
										</Button>
									</AlertDialog.Trigger>
									<AlertDialog.Content>
										<AlertDialog.Header>
											<AlertDialog.Title>{m['common.are_you_sure']()}</AlertDialog.Title>
											<AlertDialog.Description>
												{m.user_delete_desc()}</AlertDialog.Description
											>
										</AlertDialog.Header>
										<AlertDialog.Footer>
											<AlertDialog.Cancel>{m['common.cancel']()}</AlertDialog.Cancel>
											<AlertDialog.Action onclick={() => handleDelete(type.id)}
												>{m['common.confirm']()}</AlertDialog.Action
											>
										</AlertDialog.Footer>
									</AlertDialog.Content>
								</AlertDialog.Root>
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Card.Root>
	{/if}
</div>
