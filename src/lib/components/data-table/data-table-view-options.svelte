<script lang="ts" generics="TData">
	import Settings2Icon from "@lucide/svelte/icons/settings-2";
	import type { Table } from "@tanstack/table-core";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { m } from "$lib/paraglide/messages";

	let { table }: { table: Table<TData> } = $props();
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger
		class={buttonVariants({
			variant: "outline",
			size: "sm",
			class: "ml-auto hidden h-8 lg:flex bg-card",
		})}
	>
		<Settings2Icon />
		{m.view()}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content>
		<DropdownMenu.Group>
			<DropdownMenu.Label>{m.toggle_columns()}</DropdownMenu.Label>
			<DropdownMenu.Separator />
			{#each table
				.getAllColumns()
				.filter((col) => typeof col.accessorFn !== "undefined" && col.getCanHide()) as column (column)}
				<DropdownMenu.CheckboxItem
					bind:checked={() => column.getIsVisible(), (v) => column.toggleVisibility(!!v)}
					class="capitalize"
				>
					{column.id}
				</DropdownMenu.CheckboxItem>
			{/each}
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
