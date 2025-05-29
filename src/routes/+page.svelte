<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { setHeader } from '$lib/stores/state';
	import { onMount } from 'svelte';

	onMount(() => {
    setHeader({
      title: 'Scanner',
      showBackButton: false,
    });
  });

</script>

<div class="flex h-full items-center justify-center p-4 duration-500">
	<Card.Root
		class="w-[380px] backdrop-blur-sm bg-white/90 dark:bg-gray-800/90 shadow-xl hover:shadow-2xl transition-all duration-300 rounded-xl [--ring:267_100%_60%]"
	>
		<Card.Header class="space-y-2">
			<Card.Title class="text-3xl font-bold text-center bg-clip-text text-transparent">
					<p>Hello World!</p>
			</Card.Title>
		</Card.Header>
		<Card.Content class="p-6">
		</Card.Content>
	</Card.Root>
</div>
