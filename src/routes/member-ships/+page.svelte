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
	import type { MembershipType } from '$lib/models/membership_type';
	import { SpinLine } from 'svelte-loading-spinners';

	let membershipTypes: MembershipType[] = [];
	let isLoading = true;
	let error: string | null = null;

	async function fetchMembershipTypes() {
		isLoading = true;
		error = null;
		try {
			const result = await invoke<MembershipType[]>('get_all_membership_types');
			membershipTypes = result || [];
		} catch (e: any) {
			console.error('Error fetching membership types:', e);
			error = e?.message;
			toast.error(error || 'Failed to load membership types.');
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		fetchMembershipTypes();
	});

	function handleAddNew() {
		goto('/member-ships/new');
	}

	async function handleDelete(typeId: number, typeName: string) {
		try {
			await invoke('delete_membership_type', { id: typeId });
			toast.success(`Membership type "${typeName}" deleted successfully.`);
			fetchMembershipTypes();
		} catch (e: any) {
			console.error('Error deleting membership type:', e);
			toast.error(e.message || `Failed to delete ${typeName}.`);
		}
	}

	function handleEdit(typeId: number) {
		goto(`/member-ships/${typeId}`);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Membership Types</h1>
		<Button on:click={handleAddNew}>
			<PlusCircle class="mr-2 h-4 w-4" />
			Add New Type
		</Button>
	</div>

	{#if isLoading}
		<div class="absolute left-1/2 top-1/2">
			<SpinLine size="60" color="#99c1f1" unit="px" duration="2s" />
		</div>
	{:else if error}
		<Card.Root class="border-destructive">
			<Card.Header>
				<Card.Title class="text-destructive">Error</Card.Title>
			</Card.Header>
			<Card.Content>
				<p>{error}</p>
				<Button on:click={fetchMembershipTypes} variant="outline" class="mt-4">Try Again</Button>
			</Card.Content>
		</Card.Root>
	{:else if membershipTypes.length === 0}
		<Card.Root>
			<Card.Content class="pt-6">
				<p class="text-center text-muted-foreground">No membership types found.</p>
				<p class="text-center mt-2">
					<Button on:click={handleAddNew} variant="link">Add the first one!</Button>
				</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<Card.Root>
			<Card.Content>
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Name</Table.Head>
							<Table.Head>Duration</Table.Head>
							<Table.Head>Visits</Table.Head>
							<Table.Head>Enter by</Table.Head>
							<Table.Head>Description</Table.Head>
							<Table.Head class="text-right">Price</Table.Head>
							<Table.Head class="text-center">Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each membershipTypes as type (type.id)}
							<Table.Row>
								<Table.Cell class="font-medium">{type.name}</Table.Cell>
								<Table.Cell>{type.duration_days ? `${type.duration_days} days` : 'N/A'}</Table.Cell>
								<Table.Cell
									>{type.visit_limit ? `${type.visit_limit} visits` : 'Unlimited'}</Table.Cell
								>
								<Table.Cell>{type.enter_by ? `${type.enter_by}:00 h` : 'no limit'}</Table.Cell>
								<Table.Cell>{type.description ? `${type.description}` : ''}</Table.Cell>
								<Table.Cell class="text-right">${type.price.toFixed(2)}</Table.Cell>
								<Table.Cell class="text-center space-x-2">
									<Button
										on:click={() => handleEdit(type.id)}
										variant="outline"
										size="icon"
										title="Edit"
									>
										<Pencil class="h-4 w-4" />
									</Button>
									<AlertDialog.Root>
										<AlertDialog.Trigger>
											<Button variant="destructive" size="icon" title="Delete">
												<Trash2 class="h-4 w-4" />
											</Button>
										</AlertDialog.Trigger>
										<AlertDialog.Content>
											<AlertDialog.Header>
												<AlertDialog.Title>Are you absolutely sure?</AlertDialog.Title>
												<AlertDialog.Description>
													This action cannot be undone. This will permanently delete membership type
													and any user that was assigned with it will loose membership!</AlertDialog.Description
												>
											</AlertDialog.Header>
											<AlertDialog.Footer>
												<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
												<AlertDialog.Action on:click={() => handleDelete(type.id, type.name)}>Continue</AlertDialog.Action>
											</AlertDialog.Footer>
										</AlertDialog.Content>
									</AlertDialog.Root>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
