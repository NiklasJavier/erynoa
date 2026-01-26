<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: HTMLAttributes<HTMLTableSectionElement> & {
		ref?: HTMLTableSectionElement | null;
		children?: import("svelte").Snippet;
	} = $props();
</script>

<thead bind:this={ref} class={cn("[&_tr]:border-b", className)} {...restProps}>
	{@render children?.()}
</thead>
