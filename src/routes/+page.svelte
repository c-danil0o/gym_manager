<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';

	import Input from '$lib/components/ui/input/input.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import EntryStatusDialog from '$lib/components/entry-status-dialog/entry-status-dialog.svelte';

	import type { ScanProcessingResult, EntryLogDisplay } from '$lib/models/entry';
	import type { ErrorResponse } from '$lib/models/error';
	import { setHeader } from '$lib/stores/state';

	let cardIdInput = $state('');
	let inputElement: any | null = $state(null); // For focusing
	let isProcessingScan = $state(false);

	let scanResult = $state<ScanProcessingResult | null>(null);
	let showStatusDialog = $state(false);

	let recentEntries = $state<EntryLogDisplay[]>([]);
	let isLoadingEntries = $state(true);

	async function fetchRecentEntries() {
		isLoadingEntries = true;
		try {
			recentEntries = await invoke<EntryLogDisplay[]>('get_recent_entry_logs', { limit: 7 });
		} catch (e: any) {
			console.log(e);
			const errorMessage = (e as ErrorResponse)?.message || 'Failed to load recent entries!';
			toast.error(errorMessage);
		} finally {
			isLoadingEntries = false;
		}
	}

	async function handleSubmitScan() {
		if (!cardIdInput.trim()) {
			toast.info('Please enter or scan a Card ID.');
			return;
		}
		isProcessingScan = true;
		scanResult = null;

		try {
			const result = await invoke<ScanProcessingResult>('process_scan', {
				payload: { card_id: cardIdInput.trim() }
			});
			scanResult = result;
			showStatusDialog = true; // Open the dialog
			if (result.status === 'Allowed' || result.status.startsWith('Denied')) {
				fetchRecentEntries();
			}
		} catch (e: any) {
			console.log(e);
			const errorMessage =
				(e as ErrorResponse)?.message || 'Error processing scan. Please try again.';
			toast.error(errorMessage);
			scanResult = {
				status: 'Error',
				message: e.message || 'An unexpected error occurred.',
				member_name: null,
				card_id: cardIdInput.trim(),
				membership_type_name: null,
				membership_end_date: null,
				remaining_visits: null
			};
			showStatusDialog = true;
		} finally {
			isProcessingScan = false;
			cardIdInput = '';
			await tick();
			inputElement?.focus();
		}
	}

	const handleGlobalKeyPress = (event: KeyboardEvent) => {
		if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
			// Only allow numbers
			if (!/^[0-9]$/.test(event.key)) {
				return;
			}

			cardIdInput += event.key;
			inputElement?.focus();
			event.preventDefault();
		}
	};

	onMount(() => {
		setHeader({
			title: 'Scanner',
			showBackButton: false
		});

		fetchRecentEntries();
		inputElement?.focus();

		window.addEventListener('keypress', handleGlobalKeyPress);
		return () => window.removeEventListener('keypress', handleGlobalKeyPress);
	});

	$effect(() => {
		if (!showStatusDialog && inputElement) {
			setTimeout(() => inputElement?.focus(), 100);
		}
	});

	function formatDate(dateStr: string | null | undefined): string {
		if (!dateStr) return 'N/A';
		return new Date(dateStr).toLocaleString('bs-BA', {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		});
	}
</script>

<div class="space-y-20 w-full">
	<!-- Scanner Input Card -->
	<Card.Root class="mx-auto w-1/3 backdrop-blur shadow-md hover:shadow-lg">
		<Card.Content>
			<form on:submit|preventDefault={handleSubmitScan} class="flex flex-col items-center gap-4">
				<p class="text-muted-foreground p-4">Scan a member's card or enter ID and press ENTER .</p>
				<Input
					bind:this={inputElement}
					bind:value={cardIdInput}
					type="text"
					placeholder="Card ID"
					class="h-14 text-xl flex-grow"
					disabled={isProcessingScan}
					aria-label="Card ID Input"
				/>
				<Button type="submit" size="lg" class="h-14 w-1/2 m-4" disabled={isProcessingScan}>
					{#if isProcessingScan}
						Processing... <!-- Or a spinner -->
					{:else}
						Submit
					{/if}
				</Button>
			</form>
		</Card.Content>
	</Card.Root>

	<!-- Recent Entries Card -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Recent Entries</Card.Title>
		</Card.Header>
		<Card.Content>
			{#if isLoadingEntries}
				{#each Array(5) as _}
					<div class="flex items-center space-x-3 py-2.5 border-b last:border-b-0">
						<Skeleton class="h-4 w-1/4" />
						<Skeleton class="h-4 w-1/2" />
						<Skeleton class="h-4 w-1/4" />
					</div>
				{/each}
			{:else if recentEntries.length === 0}
				<p class="text-muted-foreground py-4 text-center">No recent entries.</p>
			{:else}
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Member</Table.Head>
							<Table.Head class="hidden sm:table-cell">Card ID Scanned</Table.Head>
							<Table.Head>Status</Table.Head>
							<Table.Head class="text-right">Time</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each recentEntries as entry (entry.id)}
							<Table.Row>
								<Table.Cell class="font-medium">
									{entry.member_name ? `${entry.member_name || ''}`.trim() : 'N/A'}
								</Table.Cell>
								<Table.Cell class="hidden sm:table-cell text-muted-foreground"
									>{entry.card_id || '-'}</Table.Cell
								>
								<Table.Cell>
									<Badge variant={entry.status === 'allowed' ? 'default' : 'destructive'}>
										{entry.status}
									</Badge>
								</Table.Cell>
								<Table.Cell class="text-right text-muted-foreground text-sm"
									>{formatDate(entry.entry_time)}</Table.Cell
								>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			{/if}
		</Card.Content>
	</Card.Root>
</div>

<EntryStatusDialog bind:open={showStatusDialog} autoCloseDelay={5000} result={scanResult} />
