<!--
  PasskeyLogin.svelte

  Komponente für die Passkey-Authentifizierung.
  Verwendet @erynoa/ui Design-System Komponenten.
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    activePasskeyDid,
    isPasskeyAvailable,
    isPasskeyLoading,
    passkeyCredentials,
    passkeyError,
    passkeyStore,
  } from "$lib/auth/passkey/store";
  import { formatDidShort } from "$lib/auth/passkey/utils";
  import {
    Alert,
    AlertDescription,
    AlertTitle,
  } from "@erynoa/ui/components/alert";
  import { Badge } from "@erynoa/ui/components/badge";
  import { Button } from "@erynoa/ui/components/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "@erynoa/ui/components/card";
  import {
    AlertCircle,
    Check,
    CheckCircle2,
    Fingerprint,
    Key,
    Loader2,
    Plus,
  } from "lucide-svelte";
  import { onMount } from "svelte";

  // ============================================================================
  // PROPS
  // ============================================================================

  interface Props {
    /** Callback nach erfolgreicher Authentifizierung */
    onSuccess?: (did: string) => void;
    /** Callback bei Fehler */
    onError?: (error: string) => void;
    /** Nach Login zur angegebenen URL navigieren */
    redirectTo?: string;
    /** Link zur Registrierungs-Seite */
    registerUrl?: string;
    /** Kompakte Darstellung */
    compact?: boolean;
    /** Auto-Login wenn nur ein Credential vorhanden */
    autoLogin?: boolean;
  }

  let {
    onSuccess,
    onError,
    redirectTo,
    registerUrl = "/onboarding",
    compact = false,
    autoLogin = false,
  }: Props = $props();

  // ============================================================================
  // STATE
  // ============================================================================

  let selectedCredentialId = $state<string | null>(null);
  let loginError = $state<string | null>(null);
  let loginSuccess = $state(false);

  // ============================================================================
  // LIFECYCLE
  // ============================================================================

  onMount(async () => {
    await passkeyStore.init();

    // Auto-Login wenn aktiviert und nur ein Credential
    if (autoLogin && $passkeyCredentials.length === 1) {
      selectedCredentialId = $passkeyCredentials[0].id;
      handleLogin();
    }
  });

  // ============================================================================
  // HANDLERS
  // ============================================================================

  async function handleLogin() {
    loginError = null;

    const result = await passkeyStore.authenticate({
      credentialId: selectedCredentialId || undefined,
    });

    if (result.success && result.did) {
      loginSuccess = true;
      onSuccess?.(result.did);

      if (redirectTo) {
        setTimeout(() => goto(redirectTo), 500);
      }
    } else {
      loginError = result.error || "Authentifizierung fehlgeschlagen";
      onError?.(loginError);
    }
  }

  function selectCredential(id: string) {
    selectedCredentialId = id;
  }

  function formatLastUsed(timestamp: number | undefined): string {
    if (!timestamp) return "Nie verwendet";
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return "Gerade eben";
    if (diffMins < 60) return `Vor ${diffMins} Min.`;
    if (diffMins < 1440) return `Vor ${Math.floor(diffMins / 60)} Std.`;
    return date.toLocaleDateString("de-DE");
  }
</script>

<div class="mx-auto w-full" class:max-w-sm={compact} class:max-w-md={!compact}>
  {#if !$isPasskeyAvailable}
    <!-- WebAuthn nicht verfügbar -->
    <Card class="border-destructive/50">
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-destructive/10"
        >
          <AlertCircle class="h-7 w-7 text-destructive" />
        </div>
        <CardTitle>Passkeys nicht verfügbar</CardTitle>
        <CardDescription>
          Dein Browser unterstützt keine Passkeys.
        </CardDescription>
      </CardHeader>
    </Card>
  {:else if $passkeyCredentials.length === 0}
    <!-- Keine Credentials registriert -->
    <Card>
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-muted"
        >
          <Key class="h-7 w-7 text-muted-foreground" />
        </div>
        <CardTitle>Noch kein Passkey registriert</CardTitle>
        <CardDescription>
          Erstelle zuerst eine Passkey-Identität, um dich anmelden zu können.
        </CardDescription>
      </CardHeader>
      {#if registerUrl}
        <CardContent class="pt-4">
          <Button href={registerUrl} class="w-full">
            <Plus class="mr-2 h-4 w-4" />
            Passkey erstellen
          </Button>
        </CardContent>
      {/if}
    </Card>
  {:else if loginSuccess}
    <!-- Erfolg -->
    <Card class="border-green-500/30">
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-green-500/10"
        >
          <CheckCircle2 class="h-7 w-7 text-green-500" />
        </div>
        <CardTitle>Erfolgreich angemeldet!</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="flex items-center gap-2 rounded-lg bg-muted p-3">
          <span class="text-sm text-muted-foreground">DID:</span>
          <code class="flex-1 break-all font-mono text-sm text-primary">
            {formatDidShort($activePasskeyDid || "", 28)}
          </code>
        </div>

        {#if redirectTo}
          <div
            class="flex items-center justify-center gap-2 text-sm text-muted-foreground"
          >
            <Loader2 class="h-4 w-4 animate-spin" />
            <span>Du wirst weitergeleitet...</span>
          </div>
        {/if}
      </CardContent>
    </Card>
  {:else}
    <!-- Login-Formular -->
    <Card>
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-primary/10"
        >
          <Fingerprint class="h-7 w-7 text-primary" />
        </div>
        <CardTitle>Mit Passkey anmelden</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <!-- Credential-Auswahl (wenn mehrere) -->
        {#if $passkeyCredentials.length > 1}
          <div class="space-y-2">
            <p class="text-sm text-muted-foreground">Wähle eine Identität:</p>
            {#each $passkeyCredentials as credential}
              <button
                type="button"
                class="flex w-full items-center justify-between rounded-lg border-2 p-3 text-left transition-colors
                  {selectedCredentialId === credential.id
                  ? 'border-primary bg-primary/5'
                  : 'border-border bg-muted/50 hover:border-primary/50'}"
                onclick={() => selectCredential(credential.id)}
              >
                <div class="flex flex-col gap-0.5">
                  <span class="flex items-center gap-2 font-medium">
                    {credential.displayName || "Erynoa User"}
                    {#if credential.isPrimary}
                      <Badge variant="secondary" class="text-xs">Primär</Badge>
                    {/if}
                  </span>
                  <code class="font-mono text-xs text-muted-foreground">
                    {formatDidShort(credential.did, 24)}
                  </code>
                  <span class="text-xs text-muted-foreground">
                    {formatLastUsed(credential.lastUsedAt)}
                  </span>
                </div>
                <div class="flex h-6 w-6 items-center justify-center">
                  {#if selectedCredentialId === credential.id}
                    <Check class="h-5 w-5 text-primary" />
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {:else}
          <div
            class="flex flex-col items-center gap-1 rounded-lg bg-muted/50 p-4"
          >
            <p class="text-sm text-muted-foreground">Anmelden als:</p>
            <span class="font-medium">
              {$passkeyCredentials[0]?.displayName || "Erynoa User"}
            </span>
            <code class="font-mono text-xs text-muted-foreground">
              {formatDidShort($passkeyCredentials[0]?.did || "", 28)}
            </code>
          </div>
        {/if}

        <!-- Login Button -->
        <Button
          size="lg"
          class="w-full"
          onclick={handleLogin}
          disabled={$isPasskeyLoading}
        >
          {#if $isPasskeyLoading}
            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
            Authentifiziere...
          {:else}
            <Fingerprint class="mr-2 h-4 w-4" />
            Mit Passkey anmelden
          {/if}
        </Button>

        <!-- Error -->
        {#if loginError || $passkeyError}
          <Alert variant="destructive">
            <AlertCircle class="h-4 w-4" />
            <AlertTitle>Fehler</AlertTitle>
            <AlertDescription>
              {loginError || $passkeyError}
            </AlertDescription>
          </Alert>
        {/if}

        <!-- Alternative Registrierung -->
        {#if registerUrl}
          <p class="text-center text-sm text-muted-foreground">
            Andere Identität?
            <a href={registerUrl} class="text-primary hover:underline">
              Neuen Passkey erstellen
            </a>
          </p>
        {/if}
      </CardContent>
    </Card>
  {/if}
</div>
