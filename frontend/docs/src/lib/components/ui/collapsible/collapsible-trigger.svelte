<script lang="ts">
import { cn } from '$lib/utils.js'
import { Collapsible as CollapsiblePrimitive } from 'bits-ui'
import type { Snippet } from 'svelte'
import type { TriggerProps } from './collapsible.svelte'

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: TriggerProps & { children?: Snippet } = $props()
</script>

<CollapsiblePrimitive.Trigger bind:ref class={cn(className)} {...restProps}>
	{@render children?.()}
</CollapsiblePrimitive.Trigger>
