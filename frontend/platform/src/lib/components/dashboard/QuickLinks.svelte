<script lang="ts">
  import {
    type QuickLinkGroup,
    getDashboardQuickLinksGrouped,
  } from "$lib/config/quicklinks";
  import { Badge } from "@erynoa/ui/components/badge";

  interface Props {
    class?: string;
  }

  const { class: className }: Props = $props();

  const quickLinkGroups: QuickLinkGroup[] = getDashboardQuickLinksGrouped();
</script>

<div class={className}>
  <div class="flex items-center gap-2 mb-3">
    <h2 class="text-sm font-medium text-muted-foreground">Quicklinks</h2>
  </div>

  <div class="flex flex-wrap gap-2">
    {#each quickLinkGroups as group}
      {#each group.links as link}
        <a
          href={link.url}
          class="inline-flex items-center gap-2 px-3 py-1.5 rounded-md bg-muted/50 hover:bg-muted border border-transparent hover:border-border transition-colors text-sm"
        >
          {#if link.icon}
            {@const Icon = link.icon}
            <Icon class="size-3.5 text-muted-foreground" />
          {/if}
          <span>{link.title}</span>
          <Badge
            variant="outline"
            class="text-[10px] px-1.5 py-0 h-4 font-normal"
          >
            {link.category}
          </Badge>
        </a>
      {/each}
    {/each}
  </div>
</div>
