<script lang="ts">
import { Button } from '$lib/components/ui/button'
import { cn } from '$lib/utils'
import PanelLeft from 'lucide-svelte/icons/panel-left'
import { getSidebarContext } from './sidebar-provider.svelte'

interface Props {
	class?: string
}

const { class: className }: Props = $props()
const { toggleSidebar } = getSidebarContext()
</script>

<Button
	data-sidebar="trigger"
	variant="ghost"
	size="icon"
	class={cn("h-7 w-7", className)}
	onclick={toggleSidebar}
>
	<PanelLeft class="h-4 w-4" />
	<span class="sr-only">Toggle Sidebar</span>
</Button>
