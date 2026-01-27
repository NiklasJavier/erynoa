<script lang="ts">
import { cn } from '$lib/utils'

interface Props {
	class?: string
	children?: import('svelte').Snippet
}

const { class: className, children }: Props = $props()
</script>

<div data-sidebar="content" class={cn("flex min-h-0 flex-1 flex-col gap-2 overflow-auto group-data-[collapsible=icon]:overflow-hidden", className)}>
	{@render children?.()}
</div>
