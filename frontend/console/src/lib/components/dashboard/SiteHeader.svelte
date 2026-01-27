<script lang="ts">
import { page } from '$app/stores'
import Breadcrumbs from '$lib/components/Breadcrumbs.svelte'
import { Button } from '$lib/components/ui/button'
import { Separator } from '$lib/components/ui/separator'
import * as Sidebar from '$lib/components/ui/sidebar'
import CirclePlus from 'lucide-svelte/icons/circle-plus'
</script>

<header
	class="flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)"
>
	<div class="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
		<Sidebar.Trigger class="-ms-1" />
		<Separator orientation="vertical" class="mx-2 data-[orientation=vertical]:h-4" />
		<Breadcrumbs />
		<div class="ms-auto flex items-center gap-2">
			<Button variant="ghost" size="sm" class="hidden sm:flex">
				<CirclePlus class="size-4" />
				<span>Quick Create</span>
			</Button>
		</div>
	</div>
</header>
