<script lang="ts">
	import { MembershipTypeCount } from '$lib/components/charts';
	import { setHeader } from '$lib/stores/state';
	import { invoke } from '@tauri-apps/api/core';
	import type { MembershipTypeDistribution } from '$lib/models/analytics';
	import { onMount } from 'svelte';
	let loading = $state(false);
	const chartData = $state<{ type: string; value: number; color: string }[]>([]);
	const chartConfig = $state<{
		[key: string]: { label: string; color: string };
	}>({});


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
					chartData.push({
						type: item.membership_type_name,
						value: item.active_member_count,
						color: `var(--color-${item.membership_type_name})`
					});
					chartConfig[item.membership_type_name] = {
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

	onMount(() => {
		setHeader({
			title: 'Analytics',
			showBackButton: false
		});
		fetchMembershipTypeData();
	});
</script>

<MembershipTypeCount {chartData} {chartConfig}></MembershipTypeCount>
