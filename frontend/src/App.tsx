/**
 * Main Application Component
 * Sets up routing, auth provider, and query client
 */

import { Router, Route } from "@solidjs/router";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { createSignal, onMount, Show, type ParentComponent } from "solid-js";
import { AuthProvider, useAuth } from "./lib/auth";
import { ThemeProvider } from "./lib/theme";
import { fetchConfig, type AppConfig } from "./lib/config";
import { initApiClient } from "./api/client";
import { initStorageClient } from "./api/storage";
import { Layout } from "./components/Layout";
import { ErrorBoundary } from "./components/ErrorBoundary";

// Pages
import Home from "./pages/Home";
import Users from "./pages/Users";
import Settings from "./pages/Settings";
import Callback from "./pages/Callback";
import NotFound from "./pages/NotFound";

// Create TanStack Query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60, // 1 minute
      gcTime: 1000 * 60 * 5, // 5 minutes (was cacheTime)
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  const [config, setConfig] = createSignal<AppConfig | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  onMount(async () => {
    try {
      const cfg = await fetchConfig();
      setConfig(cfg);
    } catch (err) {
      console.error("Failed to load config:", err);
      setError("Konfiguration konnte nicht geladen werden");
    }
  });

  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <ErrorBoundary>
          <Show
            when={config()}
            fallback={
              <Show
                when={error()}
                fallback={
                  <div class="flex items-center justify-center min-h-screen bg-background">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
                  </div>
                }
              >
                <div class="flex items-center justify-center min-h-screen bg-background">
                  <div class="text-center space-y-4">
                  <p class="text-destructive font-medium">{error()}</p>
                  <p class="text-muted-foreground text-sm">
                    Stelle sicher, dass das Backend läuft.
                  </p>
                </div>
              </div>
            </Show>
          }
        >
          {(cfg) => {
            return (
              <AuthProvider 
                issuer={cfg().auth.issuer} 
                clientId={cfg().auth.clientId}
              >
                <ApiInitializer>
                  <AppRouter />
                </ApiInitializer>
              </AuthProvider>
            );
          }}
        </Show>
        </ErrorBoundary>
      </QueryClientProvider>
    </ThemeProvider>
  );
}

function AppRouter() {
  return (
    <Router root={Layout}>
      <Route path="/" component={Home} />
      <Route path="/users" component={Users} />
      <Route path="/settings" component={Settings} />
      <Route path="/callback" component={Callback} />
      <Route path="*" component={NotFound} />
    </Router>
  );
}

/**
 * Initializes the API client with the auth token getter
 * Must be used inside AuthProvider
 */
const ApiInitializer: ParentComponent = (props) => {
  const auth = useAuth();

  onMount(() => {
    // Initialize API client with the auth token getter
    initApiClient(() => auth.getAccessToken());
    // Initialize Storage client with the same auth token getter
    initStorageClient(() => auth.getAccessToken());
    console.log("API and Storage clients initialized with auth token getter");
  });

  return <>{props.children}</>;
};

export default App;
