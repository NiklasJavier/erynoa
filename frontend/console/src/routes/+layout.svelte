<script lang="ts">
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import {
    isPasskeyAuthenticated,
    isPasskeyInitialized,
    isPasskeyLoading,
    passkeyError,
    passkeyStore,
  } from "$lib/auth/passkey";
  import DashboardLayout from "$lib/components/DashboardLayout.svelte";
  import { Toaster } from "@erynoa/ui/components/sonner";
  import { Loader2 } from "lucide-svelte";
  import { ModeWatcher } from "mode-watcher";
  import { onMount } from "svelte";
  import "./layout.css";

  const { children } = $props();

  // Public routes that don't require authentication (relative to base)
  const PUBLIC_ROUTES = ["/onboarding", "/callback"];

  // Check if we're on a public page (no auth required)
  // pathname includes base, so we need to check against base + route
  const isPublicPage = $derived(
    PUBLIC_ROUTES.some(
      (route) =>
        $page.url.pathname === `${base}${route}` ||
        $page.url.pathname.startsWith(`${base}${route}/`),
    ),
  );

  // Debug state visible in UI
  let debugInfo = $state("Waiting for mount...");

  // Track if we've already started redirect
  let redirecting = $state(false);

  // Initialize passkey auth in onMount to ensure we're client-side
  onMount(() => {
    debugInfo = "Mounted, initializing passkey auth...";
    console.log("[Layout] onMount triggered, initializing passkey auth...");

    passkeyStore
      .init()
      .then(() => {
        debugInfo = "Passkey auth init complete!";
        console.log("[Layout] Passkey auth init complete");
      })
      .catch((err) => {
        debugInfo = `Passkey auth init failed: ${err?.message || err}`;
        console.error("[Layout] Passkey auth init failed:", err);
      });
  });

  // Auto-redirect to onboarding when not authenticated
  $effect(() => {
    // Skip redirect check if:
    // - On public page (onboarding, callback)
    // - Still loading AND not initialized
    // - Already redirecting
    // - Not in browser
    if (
      isPublicPage ||
      (!$isPasskeyInitialized && $isPasskeyLoading) ||
      redirecting ||
      !browser
    ) {
      if (!$isPasskeyInitialized) {
        debugInfo = "Initializing passkey auth...";
      }
      return;
    }

    // Debug: Log auth state
    console.log("[Layout] Passkey auth state check:", {
      isLoading: $isPasskeyLoading,
      isInitialized: $isPasskeyInitialized,
      isAuthenticated: $isPasskeyAuthenticated,
      pathname: $page.url.pathname,
    });

    // Only redirect if we're sure user is not authenticated AND auth is initialized
    if (!$isPasskeyInitialized) {
      debugInfo = "Waiting for passkey auth initialization...";
      return;
    }

    if (!$isPasskeyAuthenticated) {
      redirecting = true;
      debugInfo = "Redirecting to onboarding...";
      console.log("[Layout] Not authenticated, redirecting to onboarding...", {
        isLoading: $isPasskeyLoading,
        isAuthenticated: $isPasskeyAuthenticated,
        isInitialized: $isPasskeyInitialized,
      });

      // Speichere aktuelle URL für Redirect nach Login (ohne base prefix)
      const currentPath = $page.url.pathname.replace(base, "") || "/";
      const returnUrl = currentPath + $page.url.search;
      if (
        returnUrl !== "/" &&
        !PUBLIC_ROUTES.some((r) => returnUrl.startsWith(r))
      ) {
        sessionStorage.setItem("auth_return_url", returnUrl);
        console.log("[Layout] Saved return URL:", returnUrl);
      }

      goto(`${base}/onboarding`);
    } else {
      // User ist authentifiziert, reset redirecting flag
      redirecting = false;
      debugInfo = "Authenticated with Passkey!";
    }
  });
</script>

<svelte:head>
  <title>Erynoa</title>
  <meta name="description" content="Erynoa - Decentralized Identity Platform" />
</svelte:head>

<ModeWatcher defaultMode="light" />

{#if isPublicPage}
  <!-- Public pages (onboarding, callback) have their own layout -->
  {@render children()}
{:else if !$isPasskeyInitialized || $isPasskeyLoading}
  <!-- Loading State (während Init) -->
  <div class="flex min-h-screen items-center justify-center bg-background">
    <div class="flex flex-col items-center gap-4">
      <Loader2 class="h-8 w-8 animate-spin text-primary" />
      <p class="text-sm text-muted-foreground">Loading...</p>
      <p class="text-xs text-muted-foreground/50">{debugInfo}</p>
      {#if $passkeyError}
        <p class="text-xs text-destructive mt-2">Error: {$passkeyError}</p>
      {/if}
    </div>
  </div>
{:else if $isPasskeyAuthenticated}
  <!-- Authenticated: Dashboard Layout -->
  <DashboardLayout>
    {@render children()}
  </DashboardLayout>
{:else}
  <!-- Not Authenticated: Redirecting to Onboarding -->
  <div class="flex min-h-screen items-center justify-center bg-background">
    <div class="flex flex-col items-center gap-4">
      <Loader2 class="h-8 w-8 animate-spin text-primary" />
      <p class="text-sm text-muted-foreground">Redirecting to onboarding...</p>
      <p class="text-xs text-muted-foreground/50">{debugInfo}</p>
    </div>
  </div>
{/if}

<Toaster />
