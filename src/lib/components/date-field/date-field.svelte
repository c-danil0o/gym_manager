<script lang="ts">
	import { DateField as DateFieldPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils';
	import { CalendarIcon } from 'lucide-svelte';
	import type { DateValue } from '@internationalized/date';

	export let value: DateValue | undefined = undefined;
	export let placeholder: DateValue | undefined = undefined;
	export let disabled = false;
	export let readonly = false;
	export let granularity: 'day' | 'hour' | 'minute' | 'second' = 'day';
	export let locale = 'en-US';
	export let hideTimeZone = false;
	export let hourCycle: 12 | 24 = 24;

	// Form integration props
	export let attrs: Record<string, any> = {};

	// Event handlers
	export let onValueChange: ((value: DateValue | undefined) => void) | undefined = undefined;
</script>

<DateFieldPrimitive.Root
	bind:value
	{placeholder}
	{disabled}
	{readonly}
	{granularity}
	{locale}
	{hideTimeZone}
	{hourCycle}
	{onValueChange}
	{...attrs}
>
	<div class="relative">
		<DateFieldPrimitive.Input
			class={cn(
				'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
				'pr-10' // Add padding for the icon
			)}
		>
			<!-- Use a snippet to get segments -->
			{#snippet children({ segments })}
				{#each segments as { part, value }}
					<DateFieldPrimitive.Segment
						{part}
						class={cn(
							'inline-block select-none rounded-sm px-1 py-0.5 focus:bg-accent hover:bg-accent focus:text-accent-foreground focus:outline-none',
							part === 'literal' ? 'text-muted-foreground' : 'text-foreground'
						)}
					>
						{value}
					</DateFieldPrimitive.Segment>
				{/each}
			{/snippet}
		</DateFieldPrimitive.Input>

		<!-- Calendar Icon -->
		<div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
			<CalendarIcon class="h-4 w-4 text-muted-foreground" />
		</div>
	</div>
</DateFieldPrimitive.Root>
