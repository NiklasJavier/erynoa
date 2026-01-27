<script lang="ts">
import { cn } from '$lib/utils'

interface Props {
	class?: string
	children?: import('svelte').Snippet
}

const { class: className, children }: Props = $props()
</script>

<div data-sidebar="group" class={cn("relative flex w-full min-w-0 flex-col px-2 py-1", className)}>
	{@render children?.()}
</div>
