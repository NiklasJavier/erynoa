<script lang="ts">
import { cn } from '../../utils.js'
import type { HTMLLabelAttributes } from 'svelte/elements'

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: HTMLLabelAttributes & {
	ref?: HTMLLabelElement | null
	children?: import('svelte').Snippet
} = $props()
</script>

<label
	bind:this={ref}
	class={cn(
		"text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</label>
