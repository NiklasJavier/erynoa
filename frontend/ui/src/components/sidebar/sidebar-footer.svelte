<script lang="ts">
import { cn } from '../../utils.js'

interface Props {
	class?: string
	children?: import('svelte').Snippet
}

const { class: className, children }: Props = $props()
</script>

<div data-sidebar="footer" class={cn("flex flex-col gap-2 p-2", className)}>
	{@render children?.()}
</div>
