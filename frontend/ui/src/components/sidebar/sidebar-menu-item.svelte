<script lang="ts">
import { cn } from '../../utils.js'

interface Props {
	class?: string
	children?: import('svelte').Snippet
}

const { class: className, children }: Props = $props()
</script>

<li data-sidebar="menu-item" class={cn("group/menu-item relative", className)}>
	{@render children?.()}
</li>
