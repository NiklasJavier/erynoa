<script lang="ts">
	import { Select as SelectPrimitive } from "bits-ui";
	import Check from "lucide-svelte/icons/check";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: SelectPrimitive.ItemProps & { children?: import("svelte").Snippet } = $props();
</script>

<SelectPrimitive.Item
	bind:ref
	class={cn(
		"relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none data-[disabled]:pointer-events-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[disabled]:opacity-50",
		className
	)}
	{...restProps}
>
	<span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
		{#snippet indicator()}
			<Check class="h-4 w-4" />
		{/snippet}
	</span>
	{@render children?.()}
</SelectPrimitive.Item>
