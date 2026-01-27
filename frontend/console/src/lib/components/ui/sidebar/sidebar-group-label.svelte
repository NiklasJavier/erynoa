<script lang="ts">
import { cn } from '$lib/utils'

interface Props {
	class?: string
	children?: import('svelte').Snippet
}

const { class: className, children }: Props = $props()
</script>

<div
	data-sidebar="group-label"
	class={cn(
		"text-sidebar-foreground/70 ring-sidebar-ring flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium outline-none transition-[margin,opa] duration-200 ease-linear focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0",
		"group-data-[collapsible=icon]:-mt-8 group-data-[collapsible=icon]:opacity-0",
		className
	)}
>
	{@render children?.()}
</div>
