<script lang="ts">
	import { Collapsible as CollapsiblePrimitive } from 'bits-ui';
	import { cn } from '$lib/utils.js';
	import type { ContentProps } from './collapsible.svelte';

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: ContentProps = $props();
</script>

<CollapsiblePrimitive.Content
	bind:ref
	class={cn(
		'data-[state=closed]:animate-collapsible-up data-[state=open]:animate-collapsible-down overflow-hidden',
		className
	)}
	{...restProps}
/>
