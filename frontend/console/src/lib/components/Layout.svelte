<script lang="ts">
import type { Snippet } from 'svelte'
import ThemeToggle from './ThemeToggle.svelte'
import UserMenu from './UserMenu.svelte'

interface Props {
	children: Snippet
}

const { children }: Props = $props()
</script>

<div class="min-h-screen bg-background">
  <!-- Header -->
  <header class="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
    <div class="container flex h-14 items-center">
      <div class="mr-4 flex">
        <a href="/" class="mr-6 flex items-center space-x-2">
          <span class="font-bold text-xl">Godstack</span>
        </a>
        <nav class="flex items-center space-x-6 text-sm font-medium">
          <a href="/" class="transition-colors hover:text-foreground/80 text-foreground/60">
            Home
          </a>
          <a href="/storage" class="transition-colors hover:text-foreground/80 text-foreground/60">
            Storage
          </a>
        </nav>
      </div>
      <div class="flex flex-1 items-center justify-end space-x-2">
        <ThemeToggle />
        <UserMenu />
      </div>
    </div>
  </header>

  <!-- Main Content -->
  <main class="container py-6">
    {@render children()}
  </main>

  <!-- Footer -->
  <footer class="border-t py-6 md:py-0">
    <div class="container flex flex-col items-center justify-between gap-4 md:h-14 md:flex-row">
      <p class="text-center text-sm leading-loose text-muted-foreground md:text-left">
        Built with SvelteKit & shadcn-svelte.
      </p>
    </div>
  </footer>
</div>
