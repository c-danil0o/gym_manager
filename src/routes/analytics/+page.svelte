<script>
	import { setHeader } from "$lib/stores/state";
	import { onMount } from "svelte";
	onMount(() => {
    setHeader({
      title: "Analytics",
      showBackButton: false
    });
  });

</script>
<p>TODO</p>
