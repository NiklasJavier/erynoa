<script lang="ts">
import { cn } from '../../utils.js'

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: {
	ref?: HTMLTableCaptionElement | null
	class?: string
	children?: import('svelte').Snippet
} & Record<string, unknown> = $props()
</script>

<caption bind:this={ref} class={cn("mt-4 text-sm text-muted-foreground", className)} {...restProps}>
	{@render children?.()}
</caption>
