<script lang="ts">
	import * as Chart from '$lib/components/ui/chart/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { PieChart, Text } from 'layerchart';
	import { m } from '$lib/paraglide/messages';

	let {
		chartData = [],
		chartConfig = {}
	}: {
		chartData: { type: string; value: number; color: string }[];
		chartConfig: Chart.ChartConfig;
	} = $props();

	const total = $derived.by(() => {
		return chartData.reduce((acc, curr) => acc + curr.value, 0);
	});
</script>

<Card.Root class="flex flex-col w-[500px] h-[500px] shrink-0">
	<Card.Header class="items-center">
		<Card.Title>{m.membership_type_distribution()}</Card.Title>
		<Card.Description>{m.membership_type_dist_desc()}</Card.Description>
	</Card.Header>
	<Card.Content class="flex-1 flex items-center">
		<Chart.Container config={chartConfig} class="mx-auto aspect-square h-[300px] ">
			<PieChart
				data={chartData}
				key="type"
				value="value"
				c="color"
				innerRadius={60}
				padding={28}
				props={{ pie: { motion: 'tween' } }}
			>
				{#snippet aboveMarks()}
					<Text
						value={String(total)}
						textAnchor="middle"
						verticalAnchor="middle"
						class="fill-foreground text-2xl! font-bold"
						style="font-size: 2rem; font-weight: bold;"
						dy={3}
					/>
					<Text
						value={m.memberships()}
						textAnchor="middle"
						verticalAnchor="middle"
						class="fill-foreground text-muted-foreground"
						dy={22}
					/>
				{/snippet}
				{#snippet tooltip()}
					<Chart.Tooltip hideLabel hideIndicator={false} />
				{/snippet}
			</PieChart>
		</Chart.Container>
	</Card.Content>
</Card.Root>
