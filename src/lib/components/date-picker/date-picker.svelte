<script lang="ts">
	import { DatePicker as DatePickerPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils';
	import { CalendarIcon, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import type { DateValue } from '@internationalized/date';

	// Main props matching your DateField component
	export let value: DateValue | undefined = undefined;
	export let placeholder: DateValue | undefined = undefined;
	export let disabled = false;
	export let readonly = false;
	export let granularity: 'day' | 'hour' | 'minute' | 'second' = 'day';
	export let locale = 'en-US';
	export let hideTimeZone = false;
	export let hourCycle: 12 | 24 = 24;
	export let height = 'h-10'; // Default height, can be overridden by parent

	// Additional DatePicker specific props
	export let open = false;
	export let closeOnSelect = true;
	export let numberOfMonths = 1;
	export let fixedWeeks = false;
	export let pagedNavigation = false;
	export let preventDeselect = false;
	export let weekStartsOn: 0 | 1 | 2 | 3 | 4 | 5 | 6 = 0;
	export let minValue: DateValue | undefined = undefined;
	export let maxValue: DateValue | undefined = undefined;

	// Form integration props
	export let attrs: Record<string, any> = {};

	// Event handlers
	export let onValueChange: ((value: DateValue | undefined) => void) | undefined = undefined;
	export let onOpenChange: ((open: boolean) => void) | undefined = undefined;
	export let onPlaceholderChange: ((placeholder: DateValue | undefined) => void) | undefined =
		undefined;

	// Calendar display props
	export let calendarLabel = 'Date picker';
	export let side: 'top' | 'right' | 'bottom' | 'left' = 'top';
	export let align: 'start' | 'center' | 'end' = 'center';
	export let sideOffset = 4;
</script>

<DatePickerPrimitive.Root
	bind:value
	bind:open
	{placeholder}
	{disabled}
	{readonly}
	{granularity}
	{locale}
	{hideTimeZone}
	{hourCycle}
	closeOnDateSelect={closeOnSelect}
	{numberOfMonths}
	{fixedWeeks}
	{pagedNavigation}
	{preventDeselect}
	{weekStartsOn}
	{minValue}
	{maxValue}
	{calendarLabel}
	{onValueChange}
	{onOpenChange}
	{onPlaceholderChange}
	{...attrs}
>
	<div class="relative">
		<DatePickerPrimitive.Input
			class={cn(
				'flex w-full rounded-md shadow border border-input bg-card px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
				'pr-10',
				height
			)}
		>
			{#snippet children({ segments })}
				{#each segments as { part, value }}
					<DatePickerPrimitive.Segment
						{part}
						class={cn(
							'inline-block select-none rounded-sm px-0.5 py-0.5 focus:bg-accent hover:bg-accent focus:text-accent-foreground focus:outline-none',
							part === 'literal' ? 'text-muted-foreground' : 'text-foreground'
						)}
					>
						{value}
					</DatePickerPrimitive.Segment>
				{/each}
			{/snippet}
		</DatePickerPrimitive.Input>

		<DatePickerPrimitive.Trigger
			class="absolute inset-y-0 right-0 flex items-center pr-3 hover:opacity-75 disabled:pointer-events-none disabled:opacity-50"
		>
			<CalendarIcon class="h-4 w-4 text-muted-foreground" />
		</DatePickerPrimitive.Trigger>
	</div>

	<DatePickerPrimitive.Content
		{side}
		{align}
		{sideOffset}
		class="z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2"
	>
		<DatePickerPrimitive.Calendar class="p-3">
			{#snippet children({ months, weekdays })}
				<DatePickerPrimitive.Header class="flex items-center justify-between pb-3">
					<DatePickerPrimitive.PrevButton
						class="inline-flex h-7 w-7 items-center justify-center rounded-sm border border-input bg-transparent p-0 opacity-50 shadow-sm hover:opacity-100 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
					>
						<ChevronLeft class="h-4 w-4" />
					</DatePickerPrimitive.PrevButton>

					<DatePickerPrimitive.Heading class="text-sm font-medium" />

					<DatePickerPrimitive.NextButton
						class="inline-flex h-7 w-7 items-center justify-center rounded-sm border border-input bg-transparent p-0 opacity-50 shadow-sm hover:opacity-100 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
					>
						<ChevronRight class="h-4 w-4" />
					</DatePickerPrimitive.NextButton>
				</DatePickerPrimitive.Header>

				<div class="flex gap-4">
					{#each months as month}
						<DatePickerPrimitive.Grid class="w-full border-collapse select-none space-y-1">
							<DatePickerPrimitive.GridHead>
								<DatePickerPrimitive.GridRow class="flex">
									{#each weekdays as day}
										<DatePickerPrimitive.HeadCell
											class="w-8 rounded-md text-[0.8rem] font-normal text-muted-foreground"
										>
											{day}
										</DatePickerPrimitive.HeadCell>
									{/each}
								</DatePickerPrimitive.GridRow>
							</DatePickerPrimitive.GridHead>

							<DatePickerPrimitive.GridBody class="space-y-1">
								{#each month.weeks as weekDates}
									<DatePickerPrimitive.GridRow class="flex w-full">
										{#each weekDates as date}
											<DatePickerPrimitive.Cell
												{date}
												month={month.value}
												class="relative h-8 w-8 rounded-md p-0 text-center text-sm focus-within:relative focus-within:z-20 [&:has([data-selected])]:bg-accent [&:has([data-selected][data-outside-month])]:bg-accent/50"
											>
												<DatePickerPrimitive.Day
													class="inline-flex h-8 w-8 items-center justify-center rounded-md text-sm font-normal transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 data-[outside-month]:text-muted-foreground data-[outside-month]:opacity-50 [&[data-outside-month][data-selected]]:bg-accent/50 [&[data-outside-month][data-selected]]:text-muted-foreground [&[data-outside-month][data-selected]]:opacity-30 data-[selected]:bg-primary data-[selected]:text-primary-foreground data-[selected]:hover:bg-primary data-[selected]:hover:text-primary-foreground data-[today]:bg-accent data-[today]:text-accent-foreground data-[selected][data-today]:bg-primary data-[selected][data-today]:text-primary-foreground data-[disabled]:text-muted-foreground data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed"
												/>
											</DatePickerPrimitive.Cell>
										{/each}
									</DatePickerPrimitive.GridRow>
								{/each}
							</DatePickerPrimitive.GridBody>
						</DatePickerPrimitive.Grid>
					{/each}
				</div>
			{/snippet}
		</DatePickerPrimitive.Calendar>
	</DatePickerPrimitive.Content>
</DatePickerPrimitive.Root>
