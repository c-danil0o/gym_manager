<script lang="ts">
	import { DatePicker } from 'bits-ui';
	import { CalendarIcon, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import {
		CalendarDate,
		DateFormatter,
		type DateValue,
		getLocalTimeZone,
		today
	} from '@internationalized/date';
	import { cn } from '$lib/utils';

	export let value: DateValue | undefined = undefined;
	export let placeholder: DateValue = today(getLocalTimeZone());
	export let disabled: boolean = false;
	export let readonly: boolean = false;
	export let locale: string = 'en-US';
	export let dateStyle: 'full' | 'long' | 'medium' | 'short' = 'medium';

	const df = new DateFormatter(locale, {
		dateStyle
	});

	let open = false;
</script>

<DatePicker.Root bind:value bind:placeholder bind:open {disabled} {readonly}>
	<DatePicker.Trigger disabled={readonly}
		class={cn(
			'flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
			$$props.class
		)}
	>
		<span class="text-sm">
			{value ? df.format(value.toDate(getLocalTimeZone())) : 'Pick a date'}
		</span>
		<CalendarIcon class="h-4 w-4 opacity-50" />
	</DatePicker.Trigger>

	<DatePicker.Content
		class="z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2"
		transitionConfig={{ duration: 100 }}
		align="start"
		sideOffset={4}
	>
		<DatePicker.Calendar
			class="p-3"
			let:months
			let:weekdays
		>
			<DatePicker.Header class="flex items-center justify-between">
				<DatePicker.PrevButton
					class="inline-flex h-7 w-7 items-center justify-center rounded-sm border border-border bg-transparent p-0 opacity-50 transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
				>
					<ChevronLeft class="h-4 w-4" />
				</DatePicker.PrevButton>
				<DatePicker.Heading class="text-sm font-medium" />
				<DatePicker.NextButton
					class="inline-flex h-7 w-7 items-center justify-center rounded-sm border border-border bg-transparent p-0 opacity-50 transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
				>
					<ChevronRight class="h-4 w-4" />
				</DatePicker.NextButton>
			</DatePicker.Header>
			<div class="flex flex-col space-y-4 pt-4 sm:flex-row sm:space-x-4 sm:space-y-0">
				{#each months as month}
					<DatePicker.Grid class="w-full border-collapse select-none space-y-1">
						<DatePicker.GridHead>
							<DatePicker.GridRow class="mb-1 flex w-full justify-center">
								{#each weekdays as day}
									<DatePicker.HeadCell
										class="h-7 w-10 rounded-md text-xs !font-normal text-muted-foreground"
									>
										<div>{day.slice(0, 2)}</div>
									</DatePicker.HeadCell>
								{/each}
							</DatePicker.GridRow>
						</DatePicker.GridHead>
						<DatePicker.GridBody>
							{#each month.weeks as weekDates}
								<DatePicker.GridRow class="flex w-full justify-center">
									{#each weekDates as date}
										<DatePicker.Cell
											{date}
											class="relative h-10 w-10 !p-0 text-center text-sm focus-within:relative focus-within:z-20"
										>
											<DatePicker.Day
												{date}
												month={month.value}
												class="inline-flex h-7 w-7 items-center justify-center rounded-md p-0 text-sm ring-offset-background transition-colors hover:bg-accent hover:text-accent-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[disabled]:text-muted-foreground data-[selected]:bg-primary data-[selected]:text-primary-foreground data-[disabled]:bg-transparent data-[unavailable]:text-destructive-foreground data-[unavailable]:line-through"
											>
												{date.day}
											</DatePicker.Day>
										</DatePicker.Cell>
									{/each}
								</DatePicker.GridRow>
							{/each}
						</DatePicker.GridBody>
					</DatePicker.Grid>
				{/each}
			</div>
		</DatePicker.Calendar>
	</DatePicker.Content>
</DatePicker.Root>
