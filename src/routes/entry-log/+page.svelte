<script>
	import { setHeader } from '$lib/stores/state';
	import { onMount } from 'svelte';
	onMount(() => {
		setHeader({
			title: 'Entry Log',
			showBackButton: false
		});
	});
</script>

<p>TODO</p>
