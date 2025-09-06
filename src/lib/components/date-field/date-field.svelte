<script lang="ts">
	import Calendar from '$lib/components/ui/calendar/calendar.svelte';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { CalendarIcon } from 'lucide-svelte';
	import { getLocalTimeZone, type DateValue } from '@internationalized/date';
	import { cn } from '$lib/utils';
	import { m } from '$lib/paraglide/messages';

	// Props using runes
	let {
		value = $bindable(),
		placeholder = undefined,
		disabled = false,
		readonly = false,
		locale = 'en-US',
		open = $bindable(false),
		minValue = undefined,
		maxValue = undefined,
		onValueChange = undefined,
		onOpenChange = undefined,
		height = 'h-10',
		class: className = '',
		...restProps
	}: {
		value?: DateValue | undefined;
		placeholder?: DateValue | undefined;
		disabled?: boolean;
		readonly?: boolean;
		locale?: string;
		open?: boolean;
		minValue?: DateValue | undefined;
		maxValue?: DateValue | undefined;
		onValueChange?: ((value: DateValue | undefined) => void) | undefined;
		onOpenChange?: ((open: boolean) => void) | undefined;
		height?: string;
		class?: string;
		[key: string]: any;
	} = $props();

	// Internal state - initialize with the prop value
	let internalValue = $state(value);
	let internalOpen = $state(open);

	// Sync internal open state with prop
	$effect(() => {
		internalOpen = open;
	});

	// Sync internal value when prop changes (external updates)
	$effect(() => {
		internalValue = value;
	});

	// Handle open state changes
	function handleOpenChange(newOpen: boolean) {
		if (readonly || disabled) return;
		internalOpen = newOpen;
		open = newOpen;
		onOpenChange?.(newOpen);
	}

	// Handle value changes from calendar selection
	function handleValueChange(newValue: DateValue | undefined) {
		if (readonly || disabled) return;
		internalValue = newValue;
		value = newValue;
		onValueChange?.(newValue);

		if (newValue) {
			internalOpen = false;
			open = false;
			onOpenChange?.(false);
		}
	}

	// Format date for display
	function formatDate(date: DateValue | undefined): string {
		if (!date) return placeholder ? formatDate(placeholder) : m.pick_date();
		try {
			return date.toDate(getLocalTimeZone()).toLocaleDateString(locale);
		} catch (error) {
			return 'Invalid date';
		}
	}
</script>

<div class={cn('relative', className)}>
	<Popover.Root bind:open={internalOpen} onOpenChange={handleOpenChange}>
		<Popover.Trigger class="w-full">
			<Button
				variant="outline"
				class={cn(
					'w-full justify-start text-left font-normal',
					!internalValue && 'text-muted-foreground',
					height
				)}
				{disabled}
				{...restProps}
			>
				<CalendarIcon class="mr-2 h-4 w-4" />
				{formatDate(internalValue)}
			</Button>
		</Popover.Trigger>
		<Popover.Content class="w-auto p-0" align="start">
			<Calendar
				value={internalValue}
				onValueChange={handleValueChange}
				{placeholder}
				{minValue}
				{maxValue}
				{locale}
				type="single"
				captionLayout="dropdown"
			/>
		</Popover.Content>
	</Popover.Root>
</div>
