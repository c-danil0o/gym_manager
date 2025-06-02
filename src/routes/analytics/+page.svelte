<script lang="ts">
	import { ActiveMembershipOT, EntryHeatmap, MembershipTypeCount } from '$lib/components/charts';
	import { setHeader } from '$lib/stores/state';
	import { invoke } from '@tauri-apps/api/core';
	import type {
		MembershipTypeDistribution,
		WeeklyHourlyDistribution,
		ActiveMembershipOverTime
	} from '$lib/models/analytics';
	import { onMount } from 'svelte';
	import { getLocalTimeZone, today, type DateValue } from '@internationalized/date';
	let loading = $state(false);

	const chartDataDist = $state<{ type: string; value: number; color: string }[]>([]);
	const chartConfigDist = $state<{
		[key: string]: { label: string; color: string };
	}>({});

	let chartDataHeatmap = $state<{ day: number; hour: number; entries: number }[]>([]);

	let chartDataActiveOT = $state<{ year_month: string; active_member_count: number }[]>([]);

	let endDate: DateValue = today(getLocalTimeZone());
	let startDate: DateValue = today(getLocalTimeZone()).subtract({ months: 12 });

	async function fetchMembershipTypeData() {
		loading = true;
		try {
			const response = await invoke<MembershipTypeDistribution[]>(
				'get_membership_type_distribution',
				{}
			);
			if (response) {
				let i = 1;
				for (const item of response) {
					chartDataDist.push({
						type: item.membership_type_name,
						value: item.active_member_count,
						color: `var(--color-${item.membership_type_name})`
					});
					chartConfigDist[item.membership_type_name] = {
						label: item.membership_type_name,
						color: `var(--chart-${i})`
					};
					i++;
				}
			} else {
				console.warn('No data received for membership type distribution');
			}
		} catch (error) {
			console.error('Failed to fetch analytics data:', error);
		} finally {
			loading = false;
		}
	}

	async function fetchWeeklyDist() {
		loading = true;
		try {
			const response = await invoke<WeeklyHourlyDistribution[]>('get_daily_hourly_visit_count', {
				startDate: startDate.toString(),
				endDate: endDate.toString()
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
		} finally {
			loading = false;
		}
	}

	async function fetchActiveOT() {
		loading = true;
		try {
			const response = await invoke<ActiveMembershipOverTime[]>(
				'get_active_memberships_over_time',
				{
					startDate: startDate.toString(),
					endDate: endDate.toString()
				}
			);
			if (response) {
				chartDataActiveOT = response;
			} else {
				console.warn('No data received for active memberships over time');
			}
		} catch (error) {
			console.error('Failed to fetch analytics data:', error);
		} finally {
			loading = false;
		}
	}

	onMount(async () => {
		setHeader({
			title: 'Analytics',
			showBackButton: false
		});
		fetchMembershipTypeData();
		fetchWeeklyDist();
		fetchActiveOT();
	});
</script>

<div class="flex flex-col gap-10">
	<div class="flex xl:flex-row flex-col gap-10 w-full xl:h-[500px]">
		<MembershipTypeCount chartData={chartDataDist} chartConfig={chartConfigDist}
		></MembershipTypeCount>
		<EntryHeatmap data={chartDataHeatmap}></EntryHeatmap>
	</div>

	<div class="flex gap-10 w-full h-[500px]">
		<ActiveMembershipOT data={chartDataActiveOT}></ActiveMembershipOT>
	</div>
</div>
