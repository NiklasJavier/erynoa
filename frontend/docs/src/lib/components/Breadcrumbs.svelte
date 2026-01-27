<script lang="ts">
import { page } from '$app/stores'
import { type BreadcrumbItem, getBreadcrumbs } from '$lib/config'
import ChevronRight from 'lucide-svelte/icons/chevron-right'

interface Props {
	/** Custom Breadcrumbs (überschreibt auto-generated) */
	items?: BreadcrumbItem[]
}

const { items }: Props = $props()

const breadcrumbs = $derived(items ?? getBreadcrumbs($page.url.pathname))
</script>

{#if breadcrumbs.length > 0}
	<nav aria-label="Breadcrumb">
		<ol class="flex items-center gap-1.5 text-sm">
			{#each breadcrumbs as crumb, i}
				<li class="flex items-center gap-1.5">
					{#if i > 0}
						<ChevronRight class="h-4 w-4 text-muted-foreground" />
					{/if}
					
					{#if crumb.isCurrentPage}
						<span class="font-medium text-foreground">
							{crumb.title}
						</span>
					{:else}
						<a 
							href={crumb.url} 
							class="text-foreground/70 transition-colors hover:text-foreground"
						>
							{crumb.title}
						</a>
					{/if}
				</li>
			{/each}
		</ol>
	</nav>
{/if}
