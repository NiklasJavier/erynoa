<script lang="ts">
  import * as Sheet from "../sheet/index.js";
  import { cn } from "../../utils.js";
  import { getSidebarContext } from "./sidebar-provider.svelte";

  interface Props {
    side?: "left" | "right";
    variant?: "sidebar" | "floating" | "inset";
    collapsible?: "offcanvas" | "icon" | "none";
    class?: string;
    children?: import("svelte").Snippet;
  }

  const {
    side = "left",
    variant = "sidebar",
    collapsible = "offcanvas",
    class: className,
    children,
  }: Props = $props();

  const { isMobile, state, openMobile, setOpenMobile } = getSidebarContext();
</script>

{#if collapsible === "none"}
  <div
    class={cn(
      "bg-sidebar text-sidebar-foreground flex h-full w-[--sidebar-width] flex-col",
      className,
    )}
    data-sidebar="sidebar"
  >
    {@render children?.()}
  </div>
{:else if $isMobile}
  <Sheet.Root open={$openMobile} onOpenChange={setOpenMobile}>
    <Sheet.Content
      class="bg-sidebar text-sidebar-foreground w-[--sidebar-width-mobile] p-0 [&>button]:hidden"
      {side}
    >
      <Sheet.Header class="sr-only">
        <Sheet.Title>Sidebar</Sheet.Title>
        <Sheet.Description>Navigation sidebar</Sheet.Description>
      </Sheet.Header>
      <div
        class="flex h-full w-full flex-col"
        data-sidebar="sidebar"
        data-mobile="true"
      >
        {@render children?.()}
      </div>
    </Sheet.Content>
  </Sheet.Root>
{:else}
  <div
    class="text-sidebar-foreground group peer hidden md:block"
    data-state={$state}
    data-collapsible={$state === "collapsed" ? collapsible : ""}
    data-variant={variant}
    data-side={side}
    data-slot="sidebar"
  >
    <!-- This is what handles the sidebar gap on desktop -->
    <div
      data-slot="sidebar-gap"
      class={cn(
        "relative w-(--sidebar-width) bg-transparent transition-[width] duration-200 ease-linear",
        "group-data-[collapsible=offcanvas]:w-0",
        "group-data-[side=right]:rotate-180",
        variant === "floating" || variant === "inset"
          ? "group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+theme(spacing.4)+2px)]"
          : "group-data-[collapsible=icon]:w-(--sidebar-width-icon)",
      )}
    ></div>
    <div
      data-slot="sidebar-container"
      class={cn(
        "fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear md:flex",
        side === "left"
          ? "start-0 group-data-[collapsible=offcanvas]:start-[calc(var(--sidebar-width)*-1)]"
          : "end-0 group-data-[collapsible=offcanvas]:end-[calc(var(--sidebar-width)*-1)]",
        variant === "floating" || variant === "inset"
          ? "p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+theme(spacing.4)+2px)]"
          : "group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-e group-data-[side=right]:border-s",
        className,
      )}
    >
      <div
        data-sidebar="sidebar"
        data-slot="sidebar-inner"
        class="bg-sidebar group-data-[variant=floating]:border-sidebar-border flex h-full w-full flex-col group-data-[variant=floating]:rounded-lg group-data-[variant=floating]:border group-data-[variant=floating]:shadow-sm"
      >
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}
