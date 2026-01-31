<script lang="ts">
import { page } from '$app/stores'
import { getDocumentTitle, getPageMeta } from '$lib/config'
import * as Alert from '@erynoa/ui/components/alert'
import { Button } from '@erynoa/ui/components/button'
import { AlertCircle, Loader2, RefreshCw } from 'lucide-svelte'
import type { Snippet } from 'svelte'

interface Props {
	/** Überschreibt Titel aus Config */
	title?: string
	/** Überschreibt Beschreibung aus Config */
	description?: string
	/** Zeigt Header (Standard: aus Config oder true) */
	showHeader?: boolean
	/** Loading State - zeigt Skeleton */
	loading?: boolean
	/** Error Message - zeigt Error Alert */
	error?: string | null
	/** Retry Callback für Error State */
	onRetry?: () => void
	/** Custom Header Actions (Buttons etc.) */
	headerActions?: Snippet
	/** Page Content */
	children: Snippet
}

const {
	title,
	description,
	showHeader,
	loading = false,
	error = null,
	onRetry,
	headerActions,
	children,
}: Props = $props()

// Auto-resolve aus Config basierend auf aktueller URL
const pageMeta = $derived(getPageMeta($page.url.pathname))
const documentTitle = $derived(getDocumentTitle($page.url.pathname))

const resolvedTitle = $derived(title ?? pageMeta?.title ?? '')
const resolvedDescription = $derived(description ?? pageMeta?.description)
const resolvedShowHeader = $derived(showHeader ?? pageMeta?.showHeader ?? true)
</script>

<!-- #2: Automatischer Document Title -->
<svelte:head>
  <title>{documentTitle}</title>
  {#if resolvedDescription}
    <meta name="description" content={resolvedDescription} />
  {/if}
</svelte:head>

<div class="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
  <div class="px-4 lg:px-6">
    {#if resolvedShowHeader && resolvedTitle}
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold tracking-tight">{resolvedTitle}</h1>
          {#if resolvedDescription}
            <p class="text-muted-foreground">{resolvedDescription}</p>
          {/if}
        </div>
        {#if headerActions && !loading && !error}
          <div class="flex items-center gap-2">
            {@render headerActions()}
          </div>
        {/if}
      </div>
    {/if}

    <!-- #4: Loading State -->
    {#if loading}
      <div class="flex flex-col items-center justify-center py-16 gap-4">
        <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
        <p class="text-sm text-muted-foreground">Loading...</p>
      </div>

      <!-- #4: Error State -->
    {:else if error}
      <Alert.Root variant="destructive" class="mb-6">
        <AlertCircle class="h-4 w-4" />
        <Alert.Title>Error</Alert.Title>
        <Alert.Description class="flex items-center justify-between">
          <span>{error}</span>
          {#if onRetry}
            <Button variant="outline" size="sm" onclick={onRetry}>
              <RefreshCw class="h-4 w-4 mr-2" />
              Retry
            </Button>
          {/if}
        </Alert.Description>
      </Alert.Root>

      <!-- Normal Content -->
    {:else}
      {@render children()}
    {/if}
  </div>
</div>
