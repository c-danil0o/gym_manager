<script lang="ts">
	import { AreaChart } from 'layerchart';
	import { curveMonotoneX } from 'd3-shape';
	import { scaleLinear } from 'd3-scale';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import * as Card from '$lib/components/ui/card/index.js';

	// Props interface
	interface ChartDataItem {
		day: number;
		hour: number;
		entries: number;
	}

	interface Props {
		data: ChartDataItem[];
		title?: string;
		description?: string;
		showAllDays?: boolean;
		colors?: string[];
		class?: string;
	}

	let {
		data,
		title = 'Entry Hours',
		description = 'Showing hourly entry patterns for each day of the week',
		showAllDays = true,
		colors = [
			'var(--chart-1)',
			'var(--chart-2)',
			'var(--chart-3)',
			'var(--chart-4)',
			'var(--chart-5)',
			'var(--chart-6)',
			'var(--chart-7)'
		],
		class: className = '',
		...restProps
	}: Props = $props();

	const dayNames = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

	// Reactive calculations using $derived
	const groupedData = $derived.by(() => {
		if (!data || data.length === 0) return [];

		// Get unique days from data
		const uniqueDays = [...new Set(data.map((d) => d.day))].sort();

		return uniqueDays
			.map((dayIndex) => {
				const dayData = data.filter((d) => d.day === dayIndex).sort((a, b) => a.hour - b.hour);

				return {
					dayName: dayNames[dayIndex] || `Day ${dayIndex}`,
					dayIndex,
					data: dayData,
					color: colors[dayIndex % colors.length]
				};
			})
			.filter((d) => d.data.length > 0);
	});

	// Create chart config reactively
	const chartConfig = $derived.by(() => {
		return groupedData.reduce((config, day) => {
			config[day.dayName.toLowerCase()] = {
				label: day.dayName,
				color: day.color
			};
			return config;
		}, {} as Chart.ChartConfig);
	});

	// Transform data for the chart reactively
	const transformedData = $derived.by(() => {
		if (groupedData.length === 0) return [];

		// Get all unique hours from the data
		const allHours = [...new Set(data.map((d) => d.hour))].sort((a, b) => a - b);
		const minHour = Math.min(...allHours);
		const maxHour = Math.max(...allHours);

		const result = [];
		for (let hour = minHour; hour <= maxHour; hour++) {
			const hourData = { hour };
			groupedData.forEach((day) => {
				const entry = day.data.find((d) => d.hour === hour);
				hourData[day.dayName.toLowerCase()] = entry ? entry.entries : 0;
			});
			result.push(hourData);
		}
		return result;
	});

	// Create series reactively
	const series = $derived.by(() => {
		return groupedData.map((day) => ({
			key: day.dayName.toLowerCase(),
			label: day.dayName,
			color: day.color
		}));
	});

	// Calculate hour range for x-axis domain
	const hourRange = $derived.by(() => {
		if (!data || data.length === 0) return [0, 23];
		const hours = data.map((d) => d.hour);
		return [Math.min(...hours), Math.max(...hours)];
	});

	// Calculate max entries for better y-axis scaling
	const maxEntries = $derived.by(() => {
		if (!data || data.length === 0) return 0;
		return Math.max(...data.map((d) => d.entries));
	});

	// Format hour for display
	function formatHour(hour: number): string {
		return `${hour.toString().padStart(2, '0')}:00`;
	}
</script>

<div class="w-full {className}" {...restProps}>
	<Card.Root class="flex flex-col w-full h-[500px] flex-shrink-0">
		<Card.Header class="text-center">
			<Card.Title>{title}</Card.Title>
			<Card.Description>{description}</Card.Description>
		</Card.Header>
		<Card.Content class="flex-1 flex items-center">
			{#if transformedData.length === 0}
				<div class="flex items-center justify-center h-32 text-muted-foreground">
					No data available
				</div>
			{:else}
				<Chart.Container config={chartConfig} class="mx-auto w-full h-[350px]">
					<AreaChart
						data={transformedData}
						x="hour"
						xScale={scaleLinear().domain(hourRange)}
						{series}
						props={{
							area: {
								curve: curveMonotoneX, // Changed from curveNatural
								'fill-opacity': 0.3,
								'stroke-width': 2,
								motion: 'tween'
							},
							xAxis: {
								format: formatHour,
								ticks: transformedData.map((d) => d.hour)
							},
							yAxis: {
								format: (v) => v.toString(),
								ticks: Array.from({ length: maxEntries + 1 }, (_, i) => i)
							}
						}}
					>
						{#snippet tooltip()}
							<Chart.Tooltip labelFormatter={(hour) => formatHour(hour)} indicator="line" />
						{/snippet}
					</AreaChart>
				</Chart.Container>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
