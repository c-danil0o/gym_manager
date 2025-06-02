<script lang="ts">
	import { AreaChart } from 'layerchart';
	import { curveMonotoneX } from 'd3-shape';
	import { scaleTime } from 'd3-scale';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import * as Card from '$lib/components/ui/card/index.js';

	// Props interface
	interface ChartDataItem {
		year_month: string; // Format: "Y-m" e.g., "2024-1", "2024-12"
		active_member_count: number;
	}

	interface Props {
		data: ChartDataItem[];
		title?: string;
		description?: string;
		class?: string;
	}

	let {
		data,
		title = 'Memberships Over Time',
		description = 'Showing active memberships count over time',
		class: className = '',
		...restProps
	}: Props = $props();

	// Transform year_month string to Date object
	function parseYearMonth(yearMonth: string): Date {
		const [year, month] = yearMonth.split('-').map(Number);
		return new Date(year, month - 1, 1); // month is 0-indexed in Date constructor
	}

	// Format date for display
	function formatMonth(date: Date): string {
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short'
		});
	}

	// Transform and sort data reactively
	const transformedData = $derived.by(() => {
		if (!data || data.length === 0) return [];

		return data
			.map(item => ({
				...item,
				date: parseYearMonth(item.year_month),
				dateValue: parseYearMonth(item.year_month).getTime()
			}))
			.sort((a, b) => a.dateValue - b.dateValue);
	});

	// Calculate date range for x-axis domain
	const dateRange = $derived.by(() => {
		if (transformedData.length === 0) return [new Date(), new Date()];
		const dates = transformedData.map(d => d.date);
		return [new Date(Math.min(...dates.map(d => d.getTime()))),
		        new Date(Math.max(...dates.map(d => d.getTime())))];
	});

	// Calculate max count for better y-axis scaling
	const maxCount = $derived.by(() => {
		if (transformedData.length === 0) return 0;
		return Math.max(...transformedData.map(d => d.active_member_count));
	});

	// Create chart config
	const chartConfig = $derived.by(() => ({
		memberships: {
			label: 'Active Memberships',
			color: 'var(--chart-1)'
		}
	} satisfies Chart.ChartConfig));

	// Create series
	const series = $derived([{
		key: 'active_member_count',
		label: 'Active Memberships',
		color: 'var(--chart-8)'
	}]);

	// Generate appropriate tick marks for x-axis
	const dateTicks = $derived.by(() => {
		if (transformedData.length === 0) return [];

		const dataCount = transformedData.length;
		if (dataCount <= 6) {
			// Show all dates if we have few data points
			return transformedData.map(d => d.date);
		} else if (dataCount <= 12) {
			// Show every other date
			return transformedData.filter((_, i) => i % 2 === 0).map(d => d.date);
		} else {
			// Show fewer ticks for many data points
			const step = Math.ceil(dataCount / 6);
			return transformedData.filter((_, i) => i % step === 0).map(d => d.date);
		}
	});

	// Generate integer ticks for y-axis
	const countTicks = $derived.by(() => {
		if (maxCount === 0) return [0];

		const tickCount = Math.min(8, maxCount + 1);
		const step = Math.ceil(maxCount / (tickCount - 1));

		const ticks = [];
		for (let i = 0; i <= maxCount; i += step) {
			ticks.push(i);
		}

		// Ensure we include the maximum value
		if (!ticks.includes(maxCount)) {
			ticks.push(maxCount);
		}

		return ticks.sort((a, b) => a - b);
	});
</script>

<div class="w-full {className}" {...restProps}>
	<Card.Root class="flex flex-col w-full h-[500px] flex-shrink-0">
		<Card.Header class="text-center">
			<Card.Title>{title}</Card.Title>
			<Card.Description>{description}</Card.Description>
		</Card.Header>
		<Card.Content class="flex-1 flex items-center mx-5">
			{#if transformedData.length === 0}
				<div class="flex items-center justify-center h-32 text-muted-foreground">
					No data available
				</div>
			{:else}
				<Chart.Container config={chartConfig} class="mx-auto w-full h-[350px]">
					<AreaChart
						data={transformedData}
						x="date"
						xScale={scaleTime().domain(dateRange)}
						{series}
						props={{
							area: {
								curve: curveMonotoneX,
								'fill-opacity': 0.3,
								'stroke-width': 2,
								motion: 'tween'
							},
							xAxis: {
								format: formatMonth,
								ticks: dateTicks
							},
							yAxis: {
								format: (v) => Math.round(v).toString(),
								ticks: countTicks
							}
						}}
					>
						{#snippet tooltip()}
							<Chart.Tooltip
								labelFormatter={(date) => formatMonth(date)}
								indicator="line"
							/>
						{/snippet}
					</AreaChart>
				</Chart.Container>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
