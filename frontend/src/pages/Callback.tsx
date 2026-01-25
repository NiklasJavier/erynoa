/**
 * OAuth Callback Page
 * Handles the redirect from ZITADEL after login
 */

import { createSignal, onMount, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";

export default function Callback() {
  const navigate = useNavigate();
  const [error, setError] = createSignal<string | null>(null);

  onMount(async () => {
    try {
      // Get the stored UserManager settings from sessionStorage
      const settingsKey = Object.keys(sessionStorage).find(k => k.startsWith("oidc."));
      
      if (!settingsKey) {
        // No OIDC state found - might be a direct navigation
        console.log("No OIDC state found, redirecting to home...");
        navigate("/", { replace: true });
        return;
      }

      // The AuthProvider should handle this, but we add a timeout fallback
      const timeout = setTimeout(() => {
        console.log("Callback timeout - redirecting to home");
        navigate("/", { replace: true });
      }, 5000);

      // Wait for AuthProvider to process
      // The actual processing happens in AuthProvider's onMount
      return () => clearTimeout(timeout);
      
    } catch (err) {
      console.error("Callback error:", err);
      setError(err instanceof Error ? err.message : "Authentication failed");
    }
  });

  return (
    <div class="flex flex-col items-center justify-center min-h-[60vh] gap-4">
      <Show
        when={!error()}
        fallback={
          <div class="text-center space-y-4">
            <p class="text-destructive font-medium">Anmeldung fehlgeschlagen</p>
            <p class="text-muted-foreground text-sm">{error()}</p>
            <button
              onClick={() => navigate("/")}
              class="text-primary hover:underline"
            >
              Zurück zur Startseite
            </button>
          </div>
        }
      >
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
        <p class="text-muted-foreground">Authentifizierung wird verarbeitet...</p>
      </Show>
    </div>
  );
}

