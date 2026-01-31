<script lang="ts">
  import { browser } from "$app/environment";
  import { page } from "$app/stores";
  import { authError, authStore, isAuthenticated, isLoading } from "$lib/auth";
  import DashboardLayout from "$lib/components/DashboardLayout.svelte";
  import { Toaster } from "@erynoa/ui/components/sonner";
  import { Loader2 } from "lucide-svelte";
  import { ModeWatcher } from "mode-watcher";
  import { onMount } from "svelte";
  import "./layout.css";

  const { children } = $props();

  // Check if we're on the callback page (could be /callback or /platform/callback)
  const isCallbackPage = $derived(
    $page.url.pathname.endsWith("/callback") ||
      $page.url.pathname === "/callback",
  );

  // Debug state visible in UI
  let debugInfo = $state("Waiting for mount...");

  // Track if we've already started redirect
  let redirecting = $state(false);

  // Track if we just came from callback (to prevent immediate redirect)
  let justFromCallback = $state(false);

  // Reaktiver Auth-Status (für Template)
  type AuthStateType = {
    isLoading: boolean;
    isInitialized: boolean;
    user: any;
  } | null;
  let authState = $state<AuthStateType>(null);

  // Subscribe to auth store for reactive updates
  $effect(() => {
    const unsubscribe = authStore.subscribe((state) => {
      authState = {
        isLoading: state.isLoading,
        isInitialized: state.isInitialized,
        user: state.user,
      };
    });
    return unsubscribe;
  });

  // Initialize auth in onMount to ensure we're client-side
  onMount(() => {
    debugInfo = "Mounted, initializing auth...";
    console.log("[Layout] onMount triggered, initializing auth...");

    authStore
      .init()
      .then(() => {
        debugInfo = "Auth init complete!";
        console.log("[Layout] Auth init complete");
      })
      .catch((err) => {
        debugInfo = `Auth init failed: ${err?.message || err}`;
        console.error("[Layout] Auth init failed:", err);
      });
  });

  // Prüfe ob wir gerade vom Callback kommen
  $effect(() => {
    const isOnCallback =
      $page.url.pathname.endsWith("/callback") ||
      $page.url.pathname === "/callback";
    if (isOnCallback) {
      justFromCallback = true;
    } else if (justFromCallback && !isOnCallback) {
      // Wir kommen vom Callback, warte kurz bevor wir prüfen
      setTimeout(() => {
        justFromCallback = false;
      }, 500);
    }
  });

  // Auto-redirect to Zitadel when not authenticated
  // Wichtig: Nicht während Callback-Verarbeitung prüfen
  $effect(() => {
    // Capture current authState value with explicit type
    const currentAuthState: AuthStateType = authState;

    // Skip redirect check if:
    // - On callback page
    // - Still loading AND not initialized (warten auf Init)
    // - Already redirecting
    // - Not in browser
    // - URL contains callback (might be processing)
    // - Just came from callback (wait for state update)
    if (
      isCallbackPage ||
      (!currentAuthState?.isInitialized && $isLoading) ||
      redirecting ||
      !browser ||
      $page.url.pathname.includes("/callback") ||
      justFromCallback
    ) {
      if (justFromCallback) {
        debugInfo = "Processing callback...";
      } else if (!currentAuthState || !currentAuthState.isInitialized) {
        debugInfo = "Initializing auth...";
      }
      return;
    }

    // Debug: Log auth state
    console.log("[Layout] Auth state check:", {
      isLoading: $isLoading,
      isInitialized: currentAuthState ? currentAuthState.isInitialized : false,
      hasUser: currentAuthState ? !!currentAuthState.user : false,
      userExpired: currentAuthState?.user?.expired ?? false,
      hasAccessToken: !!currentAuthState?.user?.access_token,
      isAuthenticated: $isAuthenticated,
      pathname: $page.url.pathname,
      justFromCallback,
    });

    // Only redirect if we're sure user is not authenticated AND auth is initialized
    if (!currentAuthState || !currentAuthState.isInitialized) {
      debugInfo = "Waiting for auth initialization...";
      return;
    }

    if (!$isAuthenticated) {
      redirecting = true;
      debugInfo = "Redirecting to Zitadel...";
      console.log("[Layout] Not authenticated, redirecting to Zitadel...", {
        isLoading: $isLoading,
        isAuthenticated: $isAuthenticated,
        isInitialized: currentAuthState
          ? currentAuthState.isInitialized
          : false,
      });

      // Speichere aktuelle URL für Redirect nach Login
      const returnUrl = $page.url.pathname + $page.url.search;
      if (returnUrl !== "/" && returnUrl !== "/callback") {
        sessionStorage.setItem("auth_return_url", returnUrl);
        console.log("[Layout] Saved return URL:", returnUrl);
      }

      authStore.login();
    } else {
      // User ist authentifiziert, reset redirecting flag
      redirecting = false;
      debugInfo = "Authenticated!";
    }
  });
</script>

<svelte:head>
  <title>Godstack</title>
  <meta name="description" content="Godstack - Modern Web Application" />
</svelte:head>

<ModeWatcher defaultMode="light" />

{#if isCallbackPage}
  <!-- Callback page has its own layout -->
  {@render children()}
{:else if !authState || !authState.isInitialized || $isLoading}
  <!-- Loading State (während Init) -->
  <div class="flex min-h-screen items-center justify-center bg-background">
    <div class="flex flex-col items-center gap-4">
      <Loader2 class="h-8 w-8 animate-spin text-primary" />
      <p class="text-sm text-muted-foreground">Loading...</p>
      <p class="text-xs text-muted-foreground/50">{debugInfo}</p>
      {#if $authError}
        <p class="text-xs text-destructive mt-2">Error: {$authError}</p>
      {/if}
    </div>
  </div>
{:else if $isAuthenticated}
  <!-- Authenticated: Dashboard Layout -->
  <DashboardLayout>
    {@render children()}
  </DashboardLayout>
{:else}
  <!-- Not Authenticated: Redirecting to Zitadel -->
  <div class="flex min-h-screen items-center justify-center bg-background">
    <div class="flex flex-col items-center gap-4">
      <Loader2 class="h-8 w-8 animate-spin text-primary" />
      <p class="text-sm text-muted-foreground">Redirecting to login...</p>
      <p class="text-xs text-muted-foreground/50">{debugInfo}</p>
    </div>
  </div>
{/if}

<Toaster />
