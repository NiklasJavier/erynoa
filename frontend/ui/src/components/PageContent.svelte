<script lang="ts">
  import * as Alert from "./alert/index.js";
  import { Button } from "./button/index.js";
  import { AlertCircle, Loader2, RefreshCw } from "lucide-svelte";
  import type { Snippet } from "svelte";

  interface Props {
    /** Page title */
    title?: string;
    /** Page description */
    description?: string;
    /** Show header section */
    showHeader?: boolean;
    /** Loading state - shows skeleton */
    loading?: boolean;
    /** Error message - shows error alert */
    error?: string | null;
    /** Retry callback for error state */
    onRetry?: () => void;
    /** Custom header actions (buttons etc.) */
    headerActions?: Snippet;
    /** Page content */
    children: Snippet;
  }

  const {
    title = "",
    description,
    showHeader = true,
    loading = false,
    error = null,
    onRetry,
    headerActions,
    children,
  }: Props = $props();
</script>

<div class="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
  <div class="px-4 lg:px-6">
    {#if showHeader && title}
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold tracking-tight lg:text-3xl">{title}</h1>
          {#if description}
            <p class="text-muted-foreground mt-1">{description}</p>
          {/if}
        </div>
        {#if headerActions}
          <div class="flex items-center gap-2">
            {@render headerActions()}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Error State -->
    {#if error}
      <Alert.Root variant="destructive" class="mb-6">
        <AlertCircle class="h-4 w-4" />
        <Alert.Title>Fehler</Alert.Title>
        <Alert.Description class="flex items-center justify-between">
          <span>{error}</span>
          {#if onRetry}
            <Button variant="outline" size="sm" onclick={onRetry}>
              <RefreshCw class="h-4 w-4 mr-2" />
              Erneut versuchen
            </Button>
          {/if}
        </Alert.Description>
      </Alert.Root>
    {/if}

    <!-- Loading State -->
    {#if loading}
      <div class="flex items-center justify-center py-12">
        <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    {:else if !error}
      {@render children()}
    {/if}
  </div>
</div>
