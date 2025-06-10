<script lang="ts">
	import { onDestroy } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { cn, translateEntryMessage } from '$lib/utils';
	import CheckCircle2 from 'lucide-svelte/icons/check-circle-2';
	import XCircle from 'lucide-svelte/icons/x-circle';
	import AlertTriangle from 'lucide-svelte/icons/alert-triangle';

	import type { ScanProcessingResult, EntryStatus } from '$lib/models/entry';
	import Label from '../ui/label/label.svelte';
	import Input from '../ui/input/input.svelte';
	import { DateFormatter } from '@internationalized/date';
	import { m } from '$lib/paraglide/messages';

	// Props
	let {
		open = $bindable(false),
		autoCloseDelay = 5000,
		result = null
	}: {
		open: boolean;
		autoCloseDelay: number;
		result?: ScanProcessingResult | null;
	} = $props();

	let countdown = $state(0);
	let timerId: number | undefined = undefined;
	let progressIntervalId: number | undefined = undefined;

	const locale = m.locale_code() || 'bs-BA';

	const df = new DateFormatter(locale, {
		dateStyle: 'long'
	});

	function startCloseTimer() {
		clearTimeout(timerId);
		clearInterval(progressIntervalId);
		countdown = autoCloseDelay / 1000;
		timerId = window.setTimeout(() => {
			open = false;
		}, autoCloseDelay);
		progressIntervalId = window.setInterval(() => {
			countdown -= 1;
			if (countdown <= 0) {
				clearInterval(progressIntervalId);
			}
		}, 1000);
	}

	// When open state changes to true from parent
	$effect(() => {
		if (open && result) {
			startCloseTimer();
			console.log(result)
		} else {
			clearTimeout(timerId);
			clearInterval(progressIntervalId);
		}
	});

	onDestroy(() => {
		clearTimeout(timerId);
		clearInterval(progressIntervalId);
	});

	function getStatusInfo(status: EntryStatus | undefined): {
		icon: typeof CheckCircle2;
		colorClass: string;
		title: string;
	} {
		switch (status) {
			case 'Allowed':
			case 'AllowedSingle':
				return {
					icon: CheckCircle2,
					colorClass: 'text-green-500',
					title: m['scanner.status_allowed']()
				};
			case 'DeniedNoMembership':
			case 'DeniedMembershipExpired':
			case 'DeniedNoVisitsLeft':
			case 'DeniedMembershipNotActiveYet':
			case 'DeniedMemberNotFound':
			case 'DeniedCardNotAssigned':
				return { icon: XCircle, colorClass: 'text-red-500', title: m['scanner.status_denied']() };
			case 'DeniedAfterHours':
			case 'DeniedAlreadyCheckedIn':
				return {
					icon: AlertTriangle,
					colorClass: 'text-yellow-500',
					title: m['scanner.status_denied']()
				};
			default:
				return {
					icon: AlertTriangle,
					colorClass: 'text-yellow-500',
					title: m['scanner.status_issue']()
				};
		}
	}
	let statusInfo = $derived(getStatusInfo(result?.status));
</script>

{#if result}
	<Dialog.Root bind:open>
		<Dialog.Content class="sm:max-w-md">
			<Dialog.Header class="items-center text-center">
				<svelte:component
					this={statusInfo.icon}
					class={cn('h-20 w-20 mb-4', statusInfo.colorClass)}
					strokeWidth={1.5}
				/>
				<Dialog.Title class={cn('text-3xl font-bold', statusInfo.colorClass)}
					>{statusInfo.title}</Dialog.Title
				>
				<Dialog.Description class="text-lg text-muted-foreground">
					{translateEntryMessage(result.message)}
				</Dialog.Description>
			</Dialog.Header>

			{#if result && result.member_name}
				<div class="mt-6 space-y-3 text-lg p-4">
					<!-- Member Name -->
					<div class="grid grid-cols-[max-content_1fr] items-center gap-x-4">
						<Label class="text-muted-foreground w-[100px]" for="dialog-member-name"
							>{m['common.member']()}:</Label
						>
						<Input
							id="dialog-member-name"
							type="text"
							value={result.member_name}
							readonly
							class="font-semibold"
						/>
					</div>

					<!-- Card ID -->
					{#if result.card_id}
						<div class="grid grid-cols-[max-content_1fr] items-center gap-x-4">
							<Label class="text-muted-foreground w-[100px]" for="dialog-card-id"
								>{m['common.card_id']()}:</Label
							>
							<Input
								id="dialog-card-id"
								type="text"
								value={result.card_id}
								readonly
								class="font-semibold"
							/>
						</div>
					{/if}

					<!-- Membership Details -->
					{#if result.membership_type_name}
						<div class="grid grid-cols-[max-content_1fr] items-center gap-x-4">
							<Label class="text-muted-foreground w-[100px]" for="dialog-membership-type"
								>{m['common.membership']()}:</Label
							>
							<Input
								id="dialog-membership-type"
								type="text"
								value={result.membership_type_name}
								readonly
								class="font-semibold"
							/>
						</div>

						{#if result.membership_end_date}
							<div class="grid grid-cols-[max-content_1fr] items-center gap-x-4">
								<Label class="text-muted-foreground w-[100px]" for="dialog-membership-end"
									>{m['scanner.ends']()}:</Label
								>
								<Input
									id="dialog-membership-end"
									type="text"
									value={df.format(new Date(result.membership_end_date))}
									readonly
									class="font-semibold"
								/>
							</div>
						{/if}

						{#if result.remaining_visits !== null && result.remaining_visits !== undefined}
							<div class="grid grid-cols-[max-content_1fr] items-center gap-x-4">
								<Label class="text-muted-foreground w-[100px]" for="dialog-remaining-visits"
									>{m['common.visits_left']()}:</Label
								>
								<Input
									id="dialog-remaining-visits"
									type="text"
									value={result.remaining_visits}
									readonly
									class="font-semibold"
								/>
							</div>
						{/if}
					{/if}
				</div>
			{/if}
			<!-- Progress bar for autoclose -->
			<div class="relative w-full h-1 bg-muted rounded-full mt-5 mb-2 overflow-hidden">
				<div
					class="absolute top-0 left-0 h-full bg-primary transition-all duration-1000 ease-linear"
					style:width="{((autoCloseDelay / 1000 - countdown) / (autoCloseDelay / 1000)) * 100}%"
				></div>
			</div>
		</Dialog.Content>
	</Dialog.Root>
{/if}
