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
	import { setHeader, setLoading } from '$lib/stores/state';
	import Input from '$lib/components/ui/input/input.svelte';
	import { m } from '$lib/paraglide/messages';
	import { enabledForRole } from '../guards';
	import Checkbox from '$lib/components/ui/checkbox/checkbox.svelte';

	let membershipTypes: MembershipType[] = [];
	let filteredMembershipTypes: MembershipType[] = [];
	let error: string | null = null;

	async function fetchMembershipTypes() {
		error = null;
		try {
			const result = await invoke<MembershipType[]>('get_all_membership_types');
			membershipTypes = result || [];
			filteredMembershipTypes = membershipTypes;
		} catch (e: any) {
			console.error('Error fetching membership types:', e);
			toast.error(m.toast_failed_membership_types());
		}
	}

	onMount(async () => {
		setHeader({
			title: m.membership_types(),
			showBackButton: false
		});
		setLoading(true);
		await fetchMembershipTypes();
		setLoading(false);
	});

	function handleAddNew() {
		goto('/memberships/new');
	}

	async function handleDelete(typeId: number, typeName: string) {
		try {
			await invoke('delete_membership_type', { id: typeId });
			toast.success(m.toast_membership_type_delete({ typeName: typeName }));
			fetchMembershipTypes();
		} catch (e: any) {
			console.error('Error deleting membership type:', e);
			toast.error(m.toast_membership_type_delete_fail());
		}
	}

	function handleEdit(typeId: number) {
		goto(`/memberships/${typeId}/edit`);
	}

	function onSearchChange(value: string) {
		if (value.trim() === '') {
			filteredMembershipTypes = membershipTypes;
		} else {
			const lowerValue = value.toLowerCase();
			filteredMembershipTypes = membershipTypes.filter((type) =>
				type.name.toLowerCase().includes(lowerValue)
			);
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
		<Button onclick={handleAddNew} class="h-8 text-xs" disabled={enabledForRole('admin')}>
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
				<Button onclick={fetchMembershipTypes} variant="outline" class="mt-4"
					>{m['common.try_again']()}</Button
				>
			</Card.Content>
		</Card.Root>
	{:else if membershipTypes.length === 0}
		<Card.Root>
			<Card.Content class="pt-6">
				<p class="text-center text-muted-foreground">{m.no_membership_types_found()}</p>
				<p class="text-center mt-2">
					<Button onclick={handleAddNew} variant="link">{m.membership_types_add_first()}</Button>
				</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<Card.Root>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>{m['common.name']()}</Table.Head>
						<Table.Head>{m['common.duration']()}</Table.Head>
						<Table.Head>{m['common.visits']()}</Table.Head>
						<Table.Head>{m['common.enter_by']()}</Table.Head>
						<Table.Head>{m['common.description']()}</Table.Head>
						<Table.Head class="text-right">{m['common.price']()}</Table.Head>
						<Table.Head class="text-center">{m.is_active()}</Table.Head>
						<Table.Head class="text-right pr-12">{m['common.actions']()}</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredMembershipTypes as type (type.id)}
						<Table.Row>
							<Table.Cell class="font-medium">{type.name}</Table.Cell>
							<Table.Cell
								>{type.duration_days
									? `${type.duration_days} ${type.duration_days === 1 ? m['common.day']() : m['common.days']()}`
									: 'N/A'}</Table.Cell
							>
							<Table.Cell
								>{type.visit_limit
									? `${type.visit_limit} ${type.visit_limit === 1 ? m['common.visit_single']() : m['common.visit_plural']()}`
									: m['common.unlimited']()}</Table.Cell
							>
							<Table.Cell
								>{type.enter_by ? `${type.enter_by}:00 h` : m['common.unlimited']()}</Table.Cell
							>
							<Table.Cell>{type.description ? `${type.description}` : ''}</Table.Cell>
							<Table.Cell class="text-right"
								>{type.price.toFixed(2)} {m.locale_currency()}</Table.Cell
							>
							<Table.Cell class="text-center"><Checkbox checked={type.is_active} /></Table.Cell>
							<Table.Cell class="text-right pr-8 space-x-2">
								<Button
									onclick={() => handleEdit(type.id)}
									variant="outline"
									size="icon"
									disabled={enabledForRole('admin')}
									title={m['common.edit']()}
								>
									<Pencil class="h-4 w-4" />
								</Button>
								<AlertDialog.Root>
									<AlertDialog.Trigger disabled={enabledForRole('admin')}>
										<Button
											variant="destructive"
											size="icon"
											disabled={enabledForRole('admin')}
											title={m['common.delete']()}
										>
											<Trash2 class="h-4 w-4" />
										</Button>
									</AlertDialog.Trigger>
									<AlertDialog.Content>
										<AlertDialog.Header>
											<AlertDialog.Title>{m['common.are_you_sure']()}</AlertDialog.Title>
											<AlertDialog.Description>
												{m.membership_delete_desc()}</AlertDialog.Description
											>
										</AlertDialog.Header>
										<AlertDialog.Footer>
											<AlertDialog.Cancel>{m['common.cancel']()}</AlertDialog.Cancel>
											<AlertDialog.Action onclick={() => handleDelete(type.id, type.name)}
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
