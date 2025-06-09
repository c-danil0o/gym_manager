<script lang="ts">
	import {
		ActiveMembershipOT,
		EntryHeatmap,
		MembershipTypeCount,
		MembershipRevenue
	} from '$lib/components/charts';
	import { setHeader, setLoading } from '$lib/stores/state';
	import { invoke } from '@tauri-apps/api/core';
	import type {
		MembershipTypeDistribution,
		WeeklyHourlyDistribution,
		ActiveMembershipOverTime,
		MembershipRevenueMap
	} from '$lib/models/analytics';
	import { onMount } from 'svelte';
	import * as Select from '$lib/components/ui/select/index.js';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';
	import Label from '$lib/components/ui/label/label.svelte';
	import { m } from '$lib/paraglide/messages';
	import { requireRole } from '../guards';

	let chartDataDist = $state<{ type: string; value: number; color: string }[]>([]);
	const chartConfigDist = $state<{
		[key: string]: { label: string; color: string };
	}>({});

	let chartDataHeatmap = $state<{ day: number; hour: number; entries: number }[]>([]);

	let chartDataActiveOT = $state<{ year_month: string; active_member_count: number }[]>([]);

	let chartDataRevenue = $state<
		{ membership_type_name: string; total_revenue: number; count: number; color: string }[]
	>([]);

	const chartConfigRevenue = $state<{
		[key: string]: { label: string; color: string };
	}>({});

	let endDate = '';
	let startDate = '';

	let currentYear = today(getLocalTimeZone()).year;
	const years: string[] = [];
	for (let i = currentYear; i >= currentYear - 5; i--) {
		years.push(String(i));
	}

	let selectedYear = $state(String(currentYear));
	let componentMounted = false;

	$effect(() => {
		let year = selectedYear;
		if (componentMounted && year) {
			loadAnalyticsData();
		}
	});

	async function loadAnalyticsData() {
		if (!selectedYear) return;
		startDate = `${selectedYear}-01-01`;
		if (String(currentYear) === selectedYear) {
			endDate = today(getLocalTimeZone()).toString();
		} else {
			endDate = `${selectedYear}-12-31`;
		}

		setLoading(true);
		try {
			await Promise.all([
				fetchMembershipTypeData(),
				fetchWeeklyDist(),
				fetchActiveOT(),
				fetchRevenueData()
			]);
		} finally {
			setLoading(false);
		}
	}

	async function fetchMembershipTypeData() {
		try {
			const response = await invoke<MembershipTypeDistribution[]>(
				'get_membership_type_distribution',
				{}
			);
			if (response) {
				let i = 1;
				const newChartDataDist = [];
				const newChartConfigDist: { [key: string]: { label: string; color: string } } = {};
				for (const item of response) {
					newChartDataDist.push({
						type: item.membership_type_name,
						value: item.active_member_count,
						color: `var(--chart-${i})`
					});
					newChartConfigDist[item.membership_type_name] = {
						label: item.membership_type_name,
						color: `var(--chart-${i})`
					};
					i++;
				}
				chartDataDist = newChartDataDist;
				for (const key in chartConfigDist) delete chartConfigDist[key];
				Object.assign(chartConfigDist, newChartConfigDist);
			} else {
				console.warn('No data received for membership type distribution');
				chartDataDist = [];
			}
		} catch (error) {
			console.error('Failed to fetch analytics data:', error);
		}
	}

	async function fetchWeeklyDist() {
		try {
			const response = await invoke<WeeklyHourlyDistribution[]>('get_daily_hourly_visit_count', {
				startDate: startDate,
				endDate: endDate
			});
			if (response) {
				chartDataHeatmap = response.map((item) => {
					return {
						day: item.day_of_week,
						hour: item.hour_of_day,
						entries: item.visit_count
					};
				});
			} else {
				console.warn('No data received for membership type distribution');
			}
		} catch (error) {
			console.error('Failed to fetch analytics data:', error);
		}
	}

	async function fetchActiveOT() {
		try {
			const response = await invoke<ActiveMembershipOverTime[]>(
				'get_active_memberships_over_time',
				{
					startDate: startDate,
					endDate: endDate
				}
			);
			if (response) {
				chartDataActiveOT = response;
			} else {
				console.warn('No data received for active memberships over time');
			}
		} catch (error) {
			console.error('Failed to fetch analytics data:', error);
		}
	}

	async function fetchRevenueData() {
		try {
			const response = await invoke<MembershipRevenueMap[]>('get_revenue_by_membership_type', {
				startDate: startDate,
				endDate: endDate
			});
			if (response) {
				let i = 1;
				const newChartDataRevenue = [];
				const newChartConfigRevenue: typeof chartConfigRevenue = {};
				for (const item of response) {
					newChartDataRevenue.push({
						membership_type_name: item.membership_type_name,
						total_revenue: item.total_revenue,
						count: item.count,
						color: `var(--chart-${i})`
					});

					newChartConfigRevenue[item.membership_type_name] = {
						label: item.membership_type_name,
						color: `var(--chart-${i})`
					};
					i++;
				}
				chartDataRevenue = newChartDataRevenue;
				for (const key in chartConfigRevenue) delete chartConfigRevenue[key];
				Object.assign(chartConfigRevenue, newChartConfigRevenue);
			} else {
				console.warn('No data received for membership revenue');
				chartDataRevenue = [];
			}
		} catch (error) {
			console.error('Failed to fetch analytics data:', error);
		}
	}

	onMount(async () => {
		requireRole('admin');
		setHeader({
			title: m['common.analytics'](),
			showBackButton: false
		});
		await loadAnalyticsData();
		componentMounted = true;
	});
</script>

<div class="flex flex-col gap-10">
	<div class="flex items-center justify-center w-full">
		<Label class="mr-4">{m.select_year()}:</Label>
		<Select.Root type="single" bind:value={selectedYear}>
			<Select.Trigger class="w-fit">
				{selectedYear ? selectedYear : m.select_year()}
			</Select.Trigger>
			<Select.Content>
				<Select.Group>
					{#each years as year}
						<Select.Item value={year} label={year}>{year}</Select.Item>
					{/each}
				</Select.Group>
			</Select.Content>
		</Select.Root>
	</div>
	<div class="flex 2xl:flex-row flex-col gap-10 w-full justify-center items-center">
		<MembershipTypeCount chartData={chartDataDist} chartConfig={chartConfigDist} />
		<EntryHeatmap data={chartDataHeatmap} />
	</div>

	<div class="flex 2xl:flex-row flex-col gap-10 w-full justify-center items-center">
		<ActiveMembershipOT data={chartDataActiveOT} />
		<MembershipRevenue chartData={chartDataRevenue} chartConfig={chartConfigRevenue} />
	</div>
</div>
