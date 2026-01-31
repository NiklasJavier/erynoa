<script lang="ts">
import { cn } from '../../utils.js'
import { Check } from 'lucide-svelte'

interface Props {
	checked?: boolean
	disabled?: boolean
	class?: string
	onchange?: (checked: boolean) => void
}

let { checked = $bindable(false), disabled = false, class: className, onchange }: Props = $props()

function toggle() {
	if (disabled) return
	checked = !checked
	onchange?.(checked)
}
</script>

<button
	type="button"
	role="checkbox"
	aria-checked={checked}
	{disabled}
	onclick={toggle}
	class={cn(
		"peer h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background",
		"focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
		"disabled:cursor-not-allowed disabled:opacity-50",
		"data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground",
		className
	)}
	data-state={checked ? "checked" : "unchecked"}
>
	{#if checked}
		<span class="flex items-center justify-center text-current">
			<Check class="h-3.5 w-3.5" />
		</span>
	{/if}
</button>
