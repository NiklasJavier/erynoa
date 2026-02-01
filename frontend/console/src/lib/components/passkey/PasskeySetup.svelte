<!--
  PasskeySetup.svelte

  Komponente für die Erstregistrierung eines Passkeys.
  Verwendet @erynoa/ui Design-System Komponenten.
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    hasPlatformAuthenticator,
    isPasskeyAvailable,
    isPasskeyLoading,
    passkeyError,
    passkeyStore,
  } from "$lib/auth/passkey/store";
  import {
    COSE_ALGORITHMS,
    type ErynoaNamespace,
    type PasskeyRegistrationOptions,
  } from "$lib/auth/passkey/types";
  import { formatDidShort } from "$lib/auth/passkey/utils";
  import {
    Alert,
    AlertDescription,
    AlertTitle,
  } from "@erynoa/ui/components/alert";
  import { Button } from "@erynoa/ui/components/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "@erynoa/ui/components/card";
  import { Checkbox } from "@erynoa/ui/components/checkbox";
  import { Input } from "@erynoa/ui/components/input";
  import { Label } from "@erynoa/ui/components/label";
  import { Separator } from "@erynoa/ui/components/separator";
  import {
    AlertCircle,
    ArrowLeft,
    ArrowRight,
    Check,
    CheckCircle2,
    Copy,
    Fingerprint,
    Key,
    Loader2,
    RefreshCw,
    Shield,
    XCircle,
  } from "lucide-svelte";
  import { onMount } from "svelte";

  // ============================================================================
  // PROPS
  // ============================================================================

  interface Props {
    /** Callback nach erfolgreicher Registrierung */
    onSuccess?: (did: string) => void;
    /** Callback bei Fehler */
    onError?: (error: string) => void;
    /** Callback bei Abbruch */
    onCancel?: () => void;
    /** Nach Registrierung zur angegebenen URL navigieren */
    redirectTo?: string;
    /** Ed25519 erzwingen (keine Fallback-Algorithmen) */
    forceEd25519?: boolean;
    /** Standard-Namespace für neue DIDs */
    defaultNamespace?: ErynoaNamespace;
    /** Kompakte Darstellung */
    compact?: boolean;
  }

  let {
    onSuccess,
    onError,
    onCancel,
    redirectTo,
    forceEd25519 = true,
    defaultNamespace = "self",
    compact = false,
  }: Props = $props();

  // ============================================================================
  // STATE
  // ============================================================================

  let displayName = $state("");
  let username = $state("");
  let preferPlatform = $state(true);
  let step = $state<"info" | "setup" | "success" | "error">("info");
  let createdDid = $state<string | null>(null);
  let setupError = $state<string | null>(null);
  let copied = $state(false);

  // ============================================================================
  // LIFECYCLE
  // ============================================================================

  onMount(async () => {
    await passkeyStore.init();
  });

  // ============================================================================
  // HANDLERS
  // ============================================================================

  function startSetup() {
    step = "setup";
  }

  function handleCancel() {
    onCancel?.();
  }

  async function handleRegister() {
    setupError = null;

    const options: PasskeyRegistrationOptions = {
      namespace: defaultNamespace,
      displayName: displayName || "Erynoa User",
      username: username || `user-${Date.now()}`,
      forceEd25519,
      preferPlatformAuthenticator: preferPlatform,
      setPrimary: true,
    };

    const result = await passkeyStore.register(options);

    if (result.success && result.did) {
      createdDid = result.did.did;
      step = "success";

      onSuccess?.(createdDid);

      if (redirectTo) {
        setTimeout(() => goto(redirectTo), 2000);
      }
    } else {
      setupError = result.error || "Passkey-Registrierung fehlgeschlagen";
      step = "error";
      onError?.(setupError);
    }
  }

  function retry() {
    setupError = null;
    step = "setup";
  }

  async function copyDid() {
    if (createdDid) {
      await navigator.clipboard.writeText(createdDid);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    }
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
          Dein Browser unterstützt keine Passkeys. Bitte verwende einen
          aktuellen Browser wie Chrome, Firefox, Safari oder Edge.
        </CardDescription>
      </CardHeader>
      {#if onCancel}
        <CardContent class="pt-4">
          <Button variant="outline" class="w-full" onclick={handleCancel}>
            <ArrowLeft class="mr-2 h-4 w-4" />
            Zurück
          </Button>
        </CardContent>
      {/if}
    </Card>
  {:else if step === "info"}
    <!-- Info-Screen -->
    <Card>
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-primary/10"
        >
          <Fingerprint class="h-7 w-7 text-primary" />
        </div>
        <CardTitle>Erstelle deine Erynoa-Identität</CardTitle>
        <CardDescription>
          Sichere, dezentrale Authentifizierung ohne Passwörter
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <!-- Features -->
        <div class="space-y-3">
          <div class="flex items-center gap-3 rounded-lg bg-muted/50 p-3">
            <Check class="h-5 w-5 shrink-0 text-green-500" />
            <span class="text-sm">Kein Passwort notwendig</span>
          </div>
          <div class="flex items-center gap-3 rounded-lg bg-muted/50 p-3">
            <Check class="h-5 w-5 shrink-0 text-green-500" />
            <span class="text-sm"
              >Gesichert durch Biometrie (TouchID, FaceID)</span
            >
          </div>
          <div class="flex items-center gap-3 rounded-lg bg-muted/50 p-3">
            <Check class="h-5 w-5 shrink-0 text-green-500" />
            <span class="text-sm">Dezentrale Identität (DID)</span>
          </div>
          <div class="flex items-center gap-3 rounded-lg bg-muted/50 p-3">
            <Check class="h-5 w-5 shrink-0 text-green-500" />
            <span class="text-sm">Du behältst die volle Kontrolle</span>
          </div>
        </div>

        <Separator />

        <!-- Authenticator Status -->
        {#if $hasPlatformAuthenticator}
          <Alert>
            <CheckCircle2 class="h-4 w-4" />
            <AlertTitle>Platform Authenticator erkannt</AlertTitle>
            <AlertDescription>
              TouchID, FaceID oder Windows Hello ist verfügbar.
            </AlertDescription>
          </Alert>
        {:else}
          <Alert>
            <Key class="h-4 w-4" />
            <AlertTitle>Security Key verwenden</AlertTitle>
            <AlertDescription>
              Kein Platform Authenticator erkannt. Du kannst einen Security Key
              (z.B. YubiKey) verwenden.
            </AlertDescription>
          </Alert>
        {/if}

        <Button class="w-full" onclick={startSetup}>
          Weiter zur Einrichtung
          <ArrowRight class="ml-2 h-4 w-4" />
        </Button>

        {#if onCancel}
          <Button variant="ghost" class="w-full" onclick={handleCancel}>
            Abbrechen
          </Button>
        {/if}
      </CardContent>
    </Card>
  {:else if step === "setup"}
    <!-- Setup-Formular -->
    <Card>
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-primary/10"
        >
          <Key class="h-7 w-7 text-primary" />
        </div>
        <CardTitle>Passkey einrichten</CardTitle>
        <CardDescription>
          Gib optionale Informationen an, um deinen Passkey zu personalisieren.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleRegister();
          }}
          class="space-y-4"
        >
          <div class="space-y-2">
            <Label for="displayName">Anzeigename (optional)</Label>
            <Input
              type="text"
              id="displayName"
              bind:value={displayName}
              placeholder="Dein Name"
              autocomplete="name"
            />
            <p class="text-xs text-muted-foreground">
              Wird nur lokal verwendet
            </p>
          </div>

          <div class="space-y-2">
            <Label for="username">Username (optional)</Label>
            <Input
              type="text"
              id="username"
              bind:value={username}
              placeholder="username"
              autocomplete="username webauthn"
            />
            <p class="text-xs text-muted-foreground">
              Für die Identifikation im Authenticator
            </p>
          </div>

          {#if $hasPlatformAuthenticator}
            <label class="flex cursor-pointer items-center space-x-2 py-2">
              <Checkbox
                checked={preferPlatform}
                onchange={(v) => (preferPlatform = v)}
              />
              <span class="text-sm font-medium leading-none">
                Biometrie bevorzugen (TouchID/FaceID)
              </span>
            </label>
          {/if}

          <div class="flex items-center gap-2 rounded-lg bg-muted/50 p-3">
            <Shield class="h-4 w-4 shrink-0 text-primary" />
            <p class="text-xs text-muted-foreground">
              Verwendet Ed25519 (alg: {COSE_ALGORITHMS.Ed25519}) für optimale
              Sicherheit.
            </p>
          </div>

          <Separator />

          <Button type="submit" class="w-full" disabled={$isPasskeyLoading}>
            {#if $isPasskeyLoading}
              <Loader2 class="mr-2 h-4 w-4 animate-spin" />
              Passkey wird erstellt...
            {:else}
              <Fingerprint class="mr-2 h-4 w-4" />
              Passkey erstellen
            {/if}
          </Button>

          <Button
            variant="ghost"
            class="w-full"
            type="button"
            onclick={() => (step = "info")}
            disabled={$isPasskeyLoading}
          >
            <ArrowLeft class="mr-2 h-4 w-4" />
            Zurück
          </Button>
        </form>
      </CardContent>
    </Card>
  {:else if step === "success"}
    <!-- Erfolg -->
    <Card class="border-green-500/30">
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-green-500/10"
        >
          <CheckCircle2 class="h-7 w-7 text-green-500" />
        </div>
        <CardTitle>Identität erstellt!</CardTitle>
        <CardDescription>
          Dein Passkey wurde erfolgreich erstellt.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        {#if createdDid}
          <div class="flex items-center gap-2 rounded-lg bg-muted p-3">
            <div class="min-w-0 flex-1">
              <p class="mb-1 text-xs text-muted-foreground">Deine DID</p>
              <code class="break-all font-mono text-sm text-primary">
                {formatDidShort(createdDid, 32)}
              </code>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onclick={copyDid}
              title="Kopieren"
            >
              {#if copied}
                <Check class="h-4 w-4 text-green-500" />
              {:else}
                <Copy class="h-4 w-4" />
              {/if}
            </Button>
          </div>
        {/if}

        <p class="text-center text-sm text-muted-foreground">
          Du kannst dich jetzt mit deiner biometrischen Authentifizierung
          anmelden.
        </p>

        {#if redirectTo}
          <div
            class="flex items-center justify-center gap-2 text-sm text-muted-foreground"
          >
            <Loader2 class="h-4 w-4 animate-spin" />
            <span>Du wirst gleich weitergeleitet...</span>
          </div>
        {:else}
          <Button class="w-full" onclick={() => goto("/")}>
            Zur Startseite
            <ArrowRight class="ml-2 h-4 w-4" />
          </Button>
        {/if}
      </CardContent>
    </Card>
  {:else if step === "error"}
    <!-- Fehler -->
    <Card class="border-destructive/30">
      <CardHeader class="pb-2 text-center">
        <div
          class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-destructive/10"
        >
          <XCircle class="h-7 w-7 text-destructive" />
        </div>
        <CardTitle>Fehler bei der Einrichtung</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>Fehler</AlertTitle>
          <AlertDescription>
            {setupError ||
              $passkeyError ||
              "Ein unbekannter Fehler ist aufgetreten."}
          </AlertDescription>
        </Alert>

        <div class="flex gap-3">
          <Button class="flex-1" onclick={retry}>
            <RefreshCw class="mr-2 h-4 w-4" />
            Erneut versuchen
          </Button>
          <Button
            variant="outline"
            class="flex-1"
            onclick={() => (step = "info")}
          >
            <ArrowLeft class="mr-2 h-4 w-4" />
            Zurück
          </Button>
        </div>
      </CardContent>
    </Card>
  {/if}
</div>
