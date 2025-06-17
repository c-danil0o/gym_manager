<script lang="ts">
	import * as m from '$lib/paraglide/messages.js';
	import { onMount, tick } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';

	import Input from '$lib/components/ui/input/input.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import EntryStatusDialog from '$lib/components/entry-status-dialog/entry-status-dialog.svelte';

	import type { ScanProcessingResult, EntryLog } from '$lib/models/entry';
	import { setHeader } from '$lib/stores/state';
	import { parseDateTime } from '@internationalized/date';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import Label from '$lib/components/ui/label/label.svelte';
	import { translateAccessStatus } from '$lib/utils';

	let cardIdInput = $state('');
	let inputElement: any | null = $state(null); // For focusing
	let isProcessingScan = $state(false);

	let first_name = $state('');
	let last_name = $state('');

	let scanResult = $state<ScanProcessingResult | null>(null);
	let showStatusDialog = $state(false);

	let recentEntries = $state<EntryLog[]>([]);
	let isLoadingEntries = $state(true);
	let activeTab = $state('membership');

	async function fetchRecentEntries() {
		isLoadingEntries = true;
		try {
			recentEntries = await invoke<EntryLog[]>('get_recent_entry_logs', { limit: 7 });
		} catch (e: any) {
			console.log(e);
			toast.error(m['scanner.toast_error_fetching_recent']());
		} finally {
			isLoadingEntries = false;
		}
	}

	async function handleSubmitScan() {
		if (!cardIdInput.trim()) {
			toast.info(m['scanner.toast_empty_card_id']());
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
			scanResult = {
				status: 'Error',
				message: m['scanner.error_unknown'](),
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

	async function handleSingleEntry() {
		if (!cardIdInput.trim() && !first_name.trim() && !last_name.trim()) {
			toast.info(m['scanner.toast_please_provide_card_id_or_name']());
			return;
		}

		isProcessingScan = true;
		scanResult = null;

		try {
			const result = await invoke<ScanProcessingResult>('process_scan_single', {
				payload: {
					card_id: cardIdInput.trim(),
					first_name: first_name.trim(),
					last_name: last_name.trim()
				}
			});
			scanResult = result;
			showStatusDialog = true; // Open the dialog
			if (result.status === 'AllowedSingle' || result.status.startsWith('Denied')) {
				fetchRecentEntries();
			}
		} catch (e: any) {
			console.log(e);
			scanResult = {
				status: 'Error',
				message: m['scanner.error_unknown'](),
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
		if (event.key === 'Enter' && !event.ctrlKey && !event.metaKey && !event.altKey) {
			// Prevent default behavior
			event.preventDefault();

			// Submit the appropriate form based on active tab
			if (activeTab === 'membership') {
				handleSubmitScan();
			} else if (activeTab === 'single') {
				handleSingleEntry();
			}
			return;
		}
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
			title: m['scanner.title'](),
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
	const locale = m.locale_code() || 'bs-BA';

	function formatDate(dateStr: string | null | undefined): string {
		if (!dateStr) return 'N/A';
		const date = parseDateTime(dateStr).toDate('UTC');
		return date.toLocaleString(locale, {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		});
	}
</script>

<div class="flex flex-col xl:mt-20 gap-10 justify-between h-full w-full">
	<!-- Scanner Input Card -->
	<Tabs.Root bind:value={activeTab} class="mx-auto w-fit">
		<Tabs.List class="grid w-full grid-cols-2 h-fit">
			<Tabs.Trigger value="membership" class="text-lg cursor-pointer">{m.membership()}</Tabs.Trigger
			>

			<Tabs.Trigger value="single" class="text-lg cursor-pointer">{m.one_time_entry()}</Tabs.Trigger
			>
		</Tabs.List>

		<Tabs.Content value="membership">
			<Card.Root class="mx-auto w-fit backdrop-blur py-2 shadow-md hover:shadow-lg">
				<Card.Content>
					<form onsubmit={handleSubmitScan} class="flex flex-col items-center gap-4">
						<Label class="text-lg font-semibold mb-2">{m['scanner.scan_card_title']()}</Label>
						<Input
							bind:ref={inputElement}
							bind:value={cardIdInput}
							type="text"
							class="h-14 text-2xl! grow text-center"
							disabled={isProcessingScan}
							aria-label={m['scanner.scan_card_title']()}
						/>
						<Separator />
						<div class="flex items-center gap-2">
							<p class="text-muted-foreground text-sm p-3">
								{m['scanner.description']()}
							</p>
							<Button type="submit" disabled={isProcessingScan}>
								{#if isProcessingScan}
									{m['scanner.processing']()}
								{:else}
									{m['scanner.scan_card_button']()}
								{/if}
							</Button>
						</div>
					</form>
				</Card.Content>
			</Card.Root>
		</Tabs.Content>

		<Tabs.Content value="single">
			<Card.Root class="mx-auto w-fit backdrop-blur py-2 shadow-md hover:shadow-lg">
				<Card.Content>
					<form onsubmit={handleSingleEntry} class="flex flex-col items-center gap-4">
						<Label class="text-lg font-semibold mb-2">{m['scanner.scan_card_title']()}</Label>
						<Input
							bind:ref={inputElement}
							bind:value={cardIdInput}
							type="text"
							class="h-14 text-2xl! grow text-center"
							disabled={isProcessingScan}
							aria-label={m['scanner.scan_card_title']()}
						/>
						<h1>Ili</h1>
						<div class="flex w-full items-center gap-6 justify-evenly">
							<div>
								<Label class="font-semibold">{m.first_name()}</Label>
								<Input type="text" bind:value={first_name} />
							</div>

							<div>
								<Label class="font-semibold">{m.last_name()}</Label>
								<Input type="text" bind:value={last_name} />
							</div>
						</div>
						<Separator />
						<div class="flex items-center gap-2">
							<p class="text-muted-foreground text-sm p-3">
								{m['scanner.single']()}
							</p>
							<Button type="submit" disabled={isProcessingScan}>
								{#if isProcessingScan}
									{m['scanner.processing']()}
								{:else}
									{m['scanner.scan_card_button']()}
								{/if}
							</Button>
						</div>
					</form>
				</Card.Content>
			</Card.Root>
		</Tabs.Content>
	</Tabs.Root>

	<!-- Recent Entries Card -->
	<Card.Root>
		<Card.Header>
			<Card.Title>{m['scanner.recent']()}</Card.Title>
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
				<p class="text-muted-foreground py-4 text-center">{m['scanner.no_recent']()}</p>
			{:else}
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>{m['common.member']()}</Table.Head>
							<Table.Head class="hidden sm:table-cell">{m['common.card_id']()}</Table.Head>
							<Table.Head>{m['common.status']()}</Table.Head>
							<Table.Head>{m['common.visits_left']()}</Table.Head>
							<Table.Head class="text-right">{m['common.time']()}</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each recentEntries as entry (entry.id)}
							<Table.Row>
								<Table.Cell class="font-medium">
									{entry.member_name ? `${entry.member_name || ''}`.trim() : '-'}
								</Table.Cell>
								<Table.Cell class="hidden sm:table-cell text-muted-foreground"
									>{entry.card_id || '-'}</Table.Cell
								>
								<Table.Cell>
									<Badge
										variant={entry.status === 'allowed' || entry.status === 'allowed_single'
											? 'default'
											: 'destructive'}
									>
										{translateAccessStatus(entry.status)}
									</Badge>
								</Table.Cell>

								<Table.Cell class="font-medium">
									{entry?.visits_left ? `${entry.visits_left || ''}`.trim() : '-'}
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

<EntryStatusDialog bind:open={showStatusDialog} autoCloseDelay={15000} result={scanResult} />
