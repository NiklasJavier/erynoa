<script lang="ts">
  /**
   * Settings Page
   *
   * Displays user identity information based on Passkey DID authentication.
   */
  import {
    activePasskeyCredential,
    activePasskeyDid,
    passkeyCredentials,
    passkeyStore,
    passkeySupport,
  } from "$lib/auth/passkey";
  import PageContent from "$lib/components/PageContent.svelte";
  import { getAppConfig } from "$lib/config";
  import { Badge } from "@erynoa/ui/components/badge";
  import { Button } from "@erynoa/ui/components/button";
  import * as Card from "@erynoa/ui/components/card";
  import { Separator } from "@erynoa/ui/components/separator";
  import {
    Bell,
    CheckCircle2,
    Fingerprint,
    Key,
    Plus,
    Server,
    Shield,
    Trash2,
    User,
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  let config = $state<ReturnType<typeof getAppConfig> | null>(null);

  onMount(() => {
    config = getAppConfig();
  });

  // Format DID for display
  function formatDid(did: string | null): string {
    if (!did) return "-";
    if (did.length <= 30) return did;
    return `${did.slice(0, 20)}...${did.slice(-8)}`;
  }

  // Format date
  function formatDate(timestamp: number | undefined): string {
    if (!timestamp) return "-";
    return new Date(timestamp).toLocaleDateString("de-DE", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  // Delete a credential
  async function deleteCredential(credentialId: string) {
    if (!confirm("Möchten Sie diesen Passkey wirklich löschen?")) return;
    passkeyStore.deleteCredential(credentialId);
    toast.success("Passkey gelöscht");
  }

  // Register a new passkey
  async function registerNewPasskey() {
    const result = await passkeyStore.register({});
    if (result.success) {
      toast.success("Neuer Passkey registriert");
    } else {
      toast.error(result.error || "Registrierung fehlgeschlagen");
    }
  }
</script>

<PageContent>
  <div class="grid gap-6 md:grid-cols-2">
    <!-- Identity Card -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <User class="h-5 w-5" />
          Identität
        </Card.Title>
        <Card.Description>Ihre dezentrale Identität (DID)</Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        <div class="space-y-2">
          <span class="text-sm font-medium">Aktive DID</span>
          <div class="p-2 bg-muted rounded-md text-sm font-mono break-all">
            {$activePasskeyDid || "-"}
          </div>
        </div>
        <div class="space-y-2">
          <span class="text-sm font-medium">Anzeigename</span>
          <div class="p-2 bg-muted rounded-md text-sm">
            {$activePasskeyCredential?.displayName || "Kein Name festgelegt"}
          </div>
        </div>
        <div class="space-y-2">
          <span class="text-sm font-medium">Erstellt am</span>
          <div class="p-2 bg-muted rounded-md text-sm">
            {formatDate($activePasskeyCredential?.createdAt)}
          </div>
        </div>
      </Card.Content>
    </Card.Root>

    <!-- System Info -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Server class="h-5 w-5" />
          System Information
        </Card.Title>
        <Card.Description>Anwendungskonfiguration und Status</Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        <div class="flex items-center justify-between">
          <span class="text-sm">Umgebung</span>
          <Badge variant="outline">{config?.environment || "-"}</Badge>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm">Version</span>
          <Badge variant="outline">v{config?.version || "-"}</Badge>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm">API URL</span>
          <span class="text-sm text-muted-foreground"
            >{config?.urls.api || "-"}</span
          >
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm">Auth</span>
          <Badge variant="secondary" class="gap-1">
            <Fingerprint class="h-3 w-3" />
            Passkey / DID
          </Badge>
        </div>
      </Card.Content>
    </Card.Root>

    <!-- Passkey Management -->
    <Card.Root class="md:col-span-2">
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Key class="h-5 w-5" />
          Passkeys
        </Card.Title>
        <Card.Description>
          Verwalten Sie Ihre registrierten Passkeys
        </Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        {#if $passkeyCredentials.length === 0}
          <div class="text-center py-8 text-muted-foreground">
            <Key class="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>Keine Passkeys registriert</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each $passkeyCredentials as credential}
              <div
                class="flex items-center justify-between p-4 rounded-lg border hover:bg-muted/50"
              >
                <div class="flex items-center gap-4">
                  <div
                    class="h-10 w-10 rounded-full bg-primary/10 flex items-center justify-center"
                  >
                    <Fingerprint class="h-5 w-5 text-primary" />
                  </div>
                  <div>
                    <div class="flex items-center gap-2">
                      <p class="font-medium">
                        {credential.displayName ||
                          `Passkey ${credential.id.slice(0, 8)}`}
                      </p>
                      {#if credential.did === $activePasskeyDid}
                        <Badge variant="default" class="text-xs gap-1">
                          <CheckCircle2 class="h-3 w-3" />
                          Aktiv
                        </Badge>
                      {/if}
                      {#if credential.isPrimary}
                        <Badge variant="secondary" class="text-xs">Primär</Badge
                        >
                      {/if}
                    </div>
                    <p class="text-sm text-muted-foreground font-mono">
                      {formatDid(credential.did)}
                    </p>
                    <p class="text-xs text-muted-foreground">
                      Erstellt: {formatDate(credential.createdAt)}
                      {#if credential.lastUsedAt}
                        • Letzte Nutzung: {formatDate(credential.lastUsedAt)}
                      {/if}
                    </p>
                  </div>
                </div>
                <div class="flex gap-2">
                  {#if credential.did !== $activePasskeyDid}
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={() => passkeyStore.setActiveDid(credential.did)}
                    >
                      Aktivieren
                    </Button>
                  {/if}
                  <Button
                    variant="ghost"
                    size="icon"
                    onclick={() => deleteCredential(credential.id)}
                    disabled={$passkeyCredentials.length === 1}
                    title={$passkeyCredentials.length === 1
                      ? "Mindestens ein Passkey erforderlich"
                      : "Passkey löschen"}
                  >
                    <Trash2 class="h-4 w-4 text-destructive" />
                  </Button>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        <Separator />

        <div class="flex justify-between items-center">
          <div>
            <p class="text-sm font-medium">Neuen Passkey hinzufügen</p>
            <p class="text-xs text-muted-foreground">
              Registrieren Sie einen weiteren Passkey für zusätzliche Sicherheit
            </p>
          </div>
          <Button
            onclick={registerNewPasskey}
            disabled={!$passkeySupport?.webauthnAvailable}
          >
            <Plus class="h-4 w-4 mr-2" />
            Passkey hinzufügen
          </Button>
        </div>
      </Card.Content>
    </Card.Root>

    <!-- Security Info -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Shield class="h-5 w-5" />
          Sicherheit
        </Card.Title>
        <Card.Description>Passkey-basierte Authentifizierung</Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium">Biometrische Authentifizierung</p>
            <p class="text-xs text-muted-foreground">
              {$passkeySupport?.platformAuthenticatorAvailable
                ? "Verfügbar auf diesem Gerät"
                : "Nicht verfügbar"}
            </p>
          </div>
          <Badge
            variant={$passkeySupport?.platformAuthenticatorAvailable
              ? "default"
              : "secondary"}
          >
            {$passkeySupport?.platformAuthenticatorAvailable ? "Aktiv" : "N/A"}
          </Badge>
        </div>
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium">Registrierte Passkeys</p>
            <p class="text-xs text-muted-foreground">
              Anzahl Ihrer Sicherheitsschlüssel
            </p>
          </div>
          <Badge variant="outline">{$passkeyCredentials.length}</Badge>
        </div>
      </Card.Content>
    </Card.Root>

    <!-- Notifications -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Bell class="h-5 w-5" />
          Benachrichtigungen
        </Card.Title>
        <Card.Description>Benachrichtigungseinstellungen</Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        <div class="flex items-center justify-center py-4 text-center">
          <div class="space-y-2">
            <p class="text-sm text-muted-foreground">
              Benachrichtigungseinstellungen demnächst verfügbar.
            </p>
          </div>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</PageContent>
