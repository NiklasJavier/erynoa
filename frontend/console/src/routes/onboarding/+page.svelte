<script lang="ts">
  /**
   * Erynoa Onboarding Page
   *
   * Guides users through identity creation with Passkey support.
   * Offers two paths:
   * 1. Traditional Ed25519 key generation
   * 2. Passkey-based DID creation (WebAuthn)
   */
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import {
    activePasskeyDid,
    isPasskeyAuthenticated,
    isPasskeyAvailableStore,
  } from "$lib/auth/passkey";
  import PasskeySetup from "$lib/components/passkey/PasskeySetup.svelte";
  import { Badge } from "@erynoa/ui/components/badge";
  import { Button } from "@erynoa/ui/components/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "@erynoa/ui/components/card";
  import { Separator } from "@erynoa/ui/components/separator";
  import {
    AlertCircle,
    ArrowRight,
    CheckCircle2,
    Fingerprint,
    Key,
    Loader2,
    Shield,
    Sparkles,
  } from "lucide-svelte";
  import { onMount } from "svelte";

  // === STATE ===
  let currentStep = $state<
    | "welcome"
    | "choose-method"
    | "passkey-setup"
    | "traditional-setup"
    | "complete"
  >("welcome");
  let passkeySupported = $state(false);
  let isSettingUp = $state(false);
  let setupError = $state<string | null>(null);
  let createdDid = $state<string | null>(null);

  // === DERIVED ===
  const isAuthenticated = $derived($isPasskeyAuthenticated);
  const activeDid = $derived($activePasskeyDid);

  // === LIFECYCLE ===
  onMount(async () => {
    // Check Passkey support
    passkeySupported = $isPasskeyAvailableStore;

    // If already authenticated, go to complete
    if ($isPasskeyAuthenticated && $activePasskeyDid) {
      createdDid = $activePasskeyDid;
      currentStep = "complete";
    }
  });

  // === METHODS ===
  function startOnboarding() {
    currentStep = "choose-method";
  }

  function choosePasskey() {
    if (passkeySupported) {
      currentStep = "passkey-setup";
    }
  }

  function chooseTraditional() {
    currentStep = "traditional-setup";
  }

  function handlePasskeySuccess(did: string) {
    createdDid = did;
    currentStep = "complete";
  }

  function handlePasskeyError(error: string) {
    setupError = error;
  }

  async function setupTraditionalIdentity() {
    isSettingUp = true;
    setupError = null;

    try {
      // TODO: Integrate with existing Ed25519 identity creation
      // For now, this is a placeholder
      await new Promise((resolve) => setTimeout(resolve, 1500));

      // Simulate DID creation
      createdDid = `did:erynoa:self:${crypto.randomUUID().slice(0, 8)}`;
      currentStep = "complete";
    } catch (err) {
      setupError =
        err instanceof Error ? err.message : "Failed to create identity";
    } finally {
      isSettingUp = false;
    }
  }

  function goToDashboard() {
    // Check if there's a saved return URL
    const returnUrl = sessionStorage.getItem("auth_return_url");
    if (returnUrl) {
      sessionStorage.removeItem("auth_return_url");
      // returnUrl is relative to base, so prepend base
      goto(`${base}${returnUrl}`);
    } else {
      goto(`${base}/`);
    }
  }

  function resetOnboarding() {
    currentStep = "welcome";
    setupError = null;
    createdDid = null;
  }
</script>

<div
  class="min-h-screen bg-gradient-to-br from-background to-muted/20 flex items-center justify-center p-4"
>
  <div class="w-full max-w-2xl">
    <!-- WELCOME STEP -->
    {#if currentStep === "welcome"}
      <Card class="border-2 border-primary/20 shadow-xl">
        <CardHeader class="text-center pb-2">
          <div
            class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-primary/10"
          >
            <Sparkles class="h-8 w-8 text-primary" />
          </div>
          <CardTitle class="text-3xl font-bold">Willkommen bei Erynoa</CardTitle
          >
          <CardDescription class="text-lg mt-2">
            Deine selbstbestimmte digitale Identität beginnt hier
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-6 pt-4">
          <div class="grid gap-4 text-sm text-muted-foreground">
            <div class="flex items-start gap-3">
              <Shield class="h-5 w-5 text-primary mt-0.5 shrink-0" />
              <div>
                <p class="font-medium text-foreground">
                  Selbstbestimmte Identität
                </p>
                <p>
                  Du kontrollierst deine Daten – keine zentrale Autorität nötig
                </p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <Key class="h-5 w-5 text-primary mt-0.5 shrink-0" />
              <div>
                <p class="font-medium text-foreground">
                  Kryptographisch sicher
                </p>
                <p>Modernste Ed25519 Kryptographie schützt deine Identität</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <Fingerprint class="h-5 w-5 text-primary mt-0.5 shrink-0" />
              <div>
                <p class="font-medium text-foreground">Passkey Support</p>
                <p>
                  Nutze biometrische Authentifizierung für maximalen Komfort
                </p>
              </div>
            </div>
          </div>

          <Separator />

          <Button size="lg" class="w-full" onclick={startOnboarding}>
            Identität erstellen
            <ArrowRight class="ml-2 h-4 w-4" />
          </Button>
        </CardContent>
      </Card>
    {/if}

    <!-- CHOOSE METHOD STEP -->
    {#if currentStep === "choose-method"}
      <Card class="border-2 shadow-xl">
        <CardHeader class="text-center">
          <CardTitle class="text-2xl"
            >Wähle deine Authentifizierungsmethode</CardTitle
          >
          <CardDescription>
            Beide Methoden erstellen eine sichere DID. Passkeys bieten
            zusätzlichen Komfort.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          {#if setupError}
            <div
              class="flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 rounded-lg"
            >
              <AlertCircle class="h-4 w-4 shrink-0" />
              <p>{setupError}</p>
            </div>
          {/if}

          <!-- Passkey Option -->
          <button
            class="w-full text-left p-4 rounded-lg border-2 transition-all hover:border-primary/50 hover:bg-muted/50 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
            class:border-primary={passkeySupported}
            class:border-muted={!passkeySupported}
            disabled={!passkeySupported}
            onclick={choosePasskey}
          >
            <div class="flex items-start gap-4">
              <div
                class="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 shrink-0"
              >
                <Fingerprint class="h-6 w-6 text-primary" />
              </div>
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <h3 class="font-semibold">Passkey (Empfohlen)</h3>
                  {#if passkeySupported}
                    <Badge variant="default" class="text-xs">Verfügbar</Badge>
                  {:else}
                    <Badge variant="secondary" class="text-xs"
                      >Nicht unterstützt</Badge
                    >
                  {/if}
                </div>
                <p class="text-sm text-muted-foreground mt-1">
                  Nutze Touch ID, Face ID oder deinen Sicherheitsschlüssel. Der
                  private Schlüssel verlässt nie dein Gerät.
                </p>
                <div class="flex flex-wrap gap-2 mt-2">
                  <Badge variant="outline" class="text-xs">Biometrisch</Badge>
                  <Badge variant="outline" class="text-xs"
                    >Hardware-geschützt</Badge
                  >
                  <Badge variant="outline" class="text-xs"
                    >Phishing-resistent</Badge
                  >
                </div>
              </div>
            </div>
          </button>

          <div class="relative">
            <div class="absolute inset-0 flex items-center">
              <span class="w-full border-t"></span>
            </div>
            <div class="relative flex justify-center text-xs uppercase">
              <span class="bg-card px-2 text-muted-foreground">oder</span>
            </div>
          </div>

          <!-- Traditional Option -->
          <button
            class="w-full text-left p-4 rounded-lg border-2 border-muted transition-all hover:border-primary/50 hover:bg-muted/50 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
            onclick={chooseTraditional}
          >
            <div class="flex items-start gap-4">
              <div
                class="flex h-12 w-12 items-center justify-center rounded-full bg-muted shrink-0"
              >
                <Key class="h-6 w-6 text-muted-foreground" />
              </div>
              <div class="flex-1">
                <h3 class="font-semibold">Traditioneller Schlüssel</h3>
                <p class="text-sm text-muted-foreground mt-1">
                  Generiere ein Ed25519 Schlüsselpaar lokal. Du bist für die
                  sichere Aufbewahrung verantwortlich.
                </p>
                <div class="flex flex-wrap gap-2 mt-2">
                  <Badge variant="outline" class="text-xs">Klassisch</Badge>
                  <Badge variant="outline" class="text-xs">Portabel</Badge>
                </div>
              </div>
            </div>
          </button>

          <Button variant="ghost" class="w-full mt-4" onclick={resetOnboarding}>
            Zurück
          </Button>
        </CardContent>
      </Card>
    {/if}

    <!-- PASSKEY SETUP STEP -->
    {#if currentStep === "passkey-setup"}
      <PasskeySetup
        onSuccess={handlePasskeySuccess}
        onError={handlePasskeyError}
        onCancel={() => (currentStep = "choose-method")}
      />
    {/if}

    <!-- TRADITIONAL SETUP STEP -->
    {#if currentStep === "traditional-setup"}
      <Card class="border-2 shadow-xl">
        <CardHeader class="text-center">
          <div
            class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-muted"
          >
            <Key class="h-7 w-7 text-muted-foreground" />
          </div>
          <CardTitle class="text-2xl">Schlüssel generieren</CardTitle>
          <CardDescription>
            Ein neues Ed25519 Schlüsselpaar wird für dich erstellt
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-6">
          {#if setupError}
            <div
              class="flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 rounded-lg"
            >
              <AlertCircle class="h-4 w-4 shrink-0" />
              <p>{setupError}</p>
            </div>
          {/if}

          <div class="space-y-4 text-sm">
            <div
              class="p-4 bg-amber-500/10 border border-amber-500/20 rounded-lg"
            >
              <p class="font-medium text-amber-700 dark:text-amber-400">
                ⚠️ Wichtiger Hinweis
              </p>
              <p class="text-muted-foreground mt-1">
                Der private Schlüssel wird lokal in deinem Browser gespeichert.
                Bei Verlust des Browserspeichers verlierst du den Zugang zu
                dieser Identität.
              </p>
            </div>
          </div>

          <div class="flex gap-3">
            <Button
              variant="outline"
              class="flex-1"
              onclick={() => (currentStep = "choose-method")}
              disabled={isSettingUp}
            >
              Zurück
            </Button>
            <Button
              class="flex-1"
              onclick={setupTraditionalIdentity}
              disabled={isSettingUp}
            >
              {#if isSettingUp}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                Generiere...
              {:else}
                Schlüssel erstellen
              {/if}
            </Button>
          </div>
        </CardContent>
      </Card>
    {/if}

    <!-- COMPLETE STEP -->
    {#if currentStep === "complete"}
      <Card class="border-2 border-green-500/30 shadow-xl">
        <CardHeader class="text-center">
          <div
            class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-500/10"
          >
            <CheckCircle2 class="h-8 w-8 text-green-500" />
          </div>
          <CardTitle class="text-2xl">Identität erstellt!</CardTitle>
          <CardDescription>
            Deine dezentrale Identität ist bereit
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-6">
          {#if createdDid}
            <div class="p-4 bg-muted rounded-lg">
              <p class="text-xs text-muted-foreground mb-1">Deine DID</p>
              <code class="text-sm font-mono break-all">{createdDid}</code>
            </div>
          {/if}

          <div class="grid gap-3 text-sm">
            <div class="flex items-center gap-2 text-muted-foreground">
              <CheckCircle2 class="h-4 w-4 text-green-500" />
              <span>Kryptographisches Schlüsselpaar generiert</span>
            </div>
            <div class="flex items-center gap-2 text-muted-foreground">
              <CheckCircle2 class="h-4 w-4 text-green-500" />
              <span>DID im Erynoa-Namespace registriert</span>
            </div>
            <div class="flex items-center gap-2 text-muted-foreground">
              <CheckCircle2 class="h-4 w-4 text-green-500" />
              <span>Lokal gespeichert und einsatzbereit</span>
            </div>
          </div>

          <Button size="lg" class="w-full" onclick={goToDashboard}>
            Zum Dashboard
            <ArrowRight class="ml-2 h-4 w-4" />
          </Button>
        </CardContent>
      </Card>
    {/if}
  </div>
</div>
