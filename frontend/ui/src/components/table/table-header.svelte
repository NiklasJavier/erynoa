<script lang="ts">
import { cn } from '../../utils.js'
import type { HTMLAttributes } from 'svelte/elements'

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: HTMLAttributes<HTMLTableSectionElement> & {
	ref?: HTMLTableSectionElement | null
	children?: import('svelte').Snippet
} = $props()
</script>

<thead bind:this={ref} class={cn("[&_tr]:border-b", className)} {...restProps}>
	{@render children?.()}
</thead>
