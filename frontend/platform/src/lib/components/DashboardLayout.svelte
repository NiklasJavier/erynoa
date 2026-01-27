<script lang="ts">
import AppSidebar from '$lib/components/dashboard/AppSidebar.svelte'
import SiteHeader from '$lib/components/dashboard/SiteHeader.svelte'
import * as Sidebar from '$lib/components/ui/sidebar'
import { getSidebarWidth } from '$lib/config'
import type { Snippet } from 'svelte'

interface Props {
	children: Snippet
}

const { children }: Props = $props()

// Dynamische Sidebar-Breite basierend auf Navigation-Tiefe
const sidebarWidth = getSidebarWidth()
</script>

<Sidebar.Provider style="--sidebar-width: {sidebarWidth}; --header-height: calc(var(--spacing) * 12);">
	<AppSidebar variant="inset" />
	<Sidebar.Inset>
		<SiteHeader />
		<div class="flex flex-1 flex-col">
			<div class="@container/main flex flex-1 flex-col gap-2">
				{@render children()}
			</div>
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
