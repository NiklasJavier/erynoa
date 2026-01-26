<script lang="ts">
	import { cn } from '$lib/utils';
	import { getSidebarContext } from './sidebar-provider.svelte';
	import { Button } from '$lib/components/ui/button';
	import PanelLeft from 'lucide-svelte/icons/panel-left';

	interface Props {
		class?: string;
	}

	let { class: className }: Props = $props();
	const { toggleSidebar } = getSidebarContext();
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
