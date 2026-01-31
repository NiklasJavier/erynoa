<script lang="ts">
import { cn } from '../../utils.js'
import type { Snippet } from 'svelte'
import type { HTMLAttributes } from 'svelte/elements'

interface Props extends HTMLAttributes<HTMLUListElement> {
	class?: string
	children?: Snippet
}

const { class: className, children, ...restProps }: Props = $props()
</script>

<ul
	class={cn(
		'mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l border-sidebar-border px-2.5 py-0.5',
		'group-data-[collapsible=icon]:hidden',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</ul>
