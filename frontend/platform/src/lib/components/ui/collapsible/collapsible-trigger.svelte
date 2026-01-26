<script lang="ts">
	import { Collapsible as CollapsiblePrimitive } from 'bits-ui';
	import { cn } from '$lib/utils.js';
	import type { TriggerProps } from './collapsible.svelte';
	import type { Snippet } from 'svelte';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: TriggerProps & { children?: Snippet } = $props();
</script>

<CollapsiblePrimitive.Trigger bind:ref class={cn(className)} {...restProps}>
	{@render children?.()}
</CollapsiblePrimitive.Trigger>
