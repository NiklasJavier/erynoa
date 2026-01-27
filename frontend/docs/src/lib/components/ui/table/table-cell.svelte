<script lang="ts">
import { cn } from '$lib/utils.js'
import type { HTMLAttributes } from 'svelte/elements'

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: HTMLAttributes<HTMLTableCellElement> & {
	ref?: HTMLTableCellElement | null
	children?: import('svelte').Snippet
} = $props()
</script>

<td
	bind:this={ref}
	class={cn("p-4 align-middle [&:has([role=checkbox])]:pr-0", className)}
	{...restProps}
>
	{@render children?.()}
</td>
