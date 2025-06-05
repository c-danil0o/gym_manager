<script lang="ts">
	import * as Chart from '$lib/components/ui/chart/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { scaleBand } from 'd3-scale';
	import { BarChart } from 'layerchart';
	import { cubicInOut } from 'svelte/easing';
	import { m } from '$lib/paraglide/messages';

	let {
		chartData = [],
		chartConfig = {}
	}: {
		chartData: { membership_type_name: string; total_revenue: number; count: number, color: string }[];
		chartConfig: Chart.ChartConfig;
	} = $props();

	const total = $derived.by(() => {
		return chartData.reduce((acc, curr) => acc + curr.total_revenue, 0);
	});
</script>

<Card.Root class="flex flex-col w-[500px] h-[500px] shrink-0">
	<Card.Header class="items-center">
		<Card.Title>{m.membership_revenue()}</Card.Title>
		<Card.Description>{m.membership_revenue_desc()}</Card.Description>
	</Card.Header>
	<Card.Content class="flex-1 flex items-center">
		<Chart.Container config={chartConfig} class="mx-auto aspect-square h-[300px] ">
			<BarChart
				data={chartData}
				orientation="horizontal"
				yScale={scaleBand().padding(0.25)}
				y="membership_type_name"
				x="total_revenue"
				cRange={chartData.map((c) => c.color)}
				c="color"
				padding={{ left: 48 }}
				grid={false}
				rule={false}
				axis="y"
				props={{
					bars: {
						stroke: 'none',
						radius: 5,
						rounded: 'all',
						initialWidth: 0,
						initialX: 0,
						motion: {
							x: { type: 'tween', duration: 500, easing: cubicInOut },
							width: { type: 'tween', duration: 500, easing: cubicInOut }
						}
					},
					highlight: { area: { fill: 'none' } },
					yAxis: {
						format: (d) => chartConfig[d as keyof typeof chartConfig].label,
						tickLabelProps: {
							svgProps: {
								x: -16
							}
						}
					}
				}}
			>
				{#snippet tooltip()}
					<Chart.Tooltip
				 />
				{/snippet}
			</BarChart>
		</Chart.Container>
	</Card.Content>
	<Card.Footer class="flex flex-col items-center justify-center gap-2">
    <div class="text-muted-foreground text-sm">{m.total_revenue()}</div>
    <div class="text-2xl font-bold">{total.toLocaleString('en-US', { style: 'currency', currency: m.currency() })}</div>
	</Card.Footer>
</Card.Root>
