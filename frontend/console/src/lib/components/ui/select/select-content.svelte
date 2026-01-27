<script lang="ts">
import { cn } from '$lib/utils.js'
import { Select as SelectPrimitive } from 'bits-ui'
import ChevronDown from 'lucide-svelte/icons/chevron-down'
import ChevronUp from 'lucide-svelte/icons/chevron-up'

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: SelectPrimitive.ContentProps = $props()
</script>

<SelectPrimitive.Portal>
	<SelectPrimitive.Content
		bind:ref
		class={cn(
			"relative z-50 max-h-96 min-w-[8rem] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
			className
		)}
		{...restProps}
	>
		<SelectPrimitive.ScrollUpButton class="flex cursor-default items-center justify-center py-1">
			<ChevronUp class="h-4 w-4" />
		</SelectPrimitive.ScrollUpButton>
		<SelectPrimitive.Viewport class="p-1">
			{@render children?.()}
		</SelectPrimitive.Viewport>
		<SelectPrimitive.ScrollDownButton class="flex cursor-default items-center justify-center py-1">
			<ChevronDown class="h-4 w-4" />
		</SelectPrimitive.ScrollDownButton>
	</SelectPrimitive.Content>
</SelectPrimitive.Portal>
