<script lang="ts">
  import ChevronRight from "lucide-svelte/icons/chevron-right";

  export interface BreadcrumbItem {
    title: string;
    url?: string;
    isCurrentPage?: boolean;
  }

  interface Props {
    /** Breadcrumb items to display */
    items: BreadcrumbItem[];
  }

  const { items }: Props = $props();
</script>

{#if items.length > 0}
  <nav aria-label="Breadcrumb">
    <ol class="flex items-center gap-1.5 text-sm">
      {#each items as crumb, i}
        <li class="flex items-center gap-1.5">
          {#if i > 0}
            <ChevronRight class="h-4 w-4 text-muted-foreground" />
          {/if}

          {#if crumb.isCurrentPage || !crumb.url}
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
