<!--
  PasskeyManager.svelte

  Komponente zur Verwaltung registrierter Passkeys.
  Verwendet @erynoa/ui Design-System Komponenten.
-->
<script lang="ts">
  import {
    activePasskeyDid,
    passkeyCredentials,
    passkeyStore,
  } from "$lib/auth/passkey/store";
  import { COSE_ALGORITHMS } from "$lib/auth/passkey/types";
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
    CardHeader,
    CardTitle,
  } from "@erynoa/ui/components/card";
  import { Separator } from "@erynoa/ui/components/separator";
  import {
    AlertTriangle,
    Check,
    ChevronDown,
    ChevronUp,
    Copy,
    Key,
    Trash2,
    X,
  } from "lucide-svelte";
  import { onMount } from "svelte";

  // ============================================================================
  // PROPS
  // ============================================================================

  interface Props {
    /** Callback wenn ein Passkey gelöscht wird */
    onDelete?: (credentialId: string) => void;
    /** Callback wenn ein Passkey aktiviert wird */
    onActivate?: (did: string) => void;
    /** Kompakte Darstellung */
    compact?: boolean;
    /** Löschung bestätigen */
    confirmDelete?: boolean;
  }

  let {
    onDelete,
    onActivate,
    compact = false,
    confirmDelete = true,
  }: Props = $props();

  // ============================================================================
  // STATE
  // ============================================================================

  let deleteConfirmId = $state<string | null>(null);
  let expandedId = $state<string | null>(null);
  let copiedId = $state<string | null>(null);

  // ============================================================================
  // LIFECYCLE
  // ============================================================================

  onMount(async () => {
    await passkeyStore.init();
  });

  // ============================================================================
  // HANDLERS
  // ============================================================================

  function handleActivate(did: string) {
    passkeyStore.setActiveDid(did);
    onActivate?.(did);
  }

  function initiateDelete(credentialId: string) {
    if (confirmDelete) {
      deleteConfirmId = credentialId;
    } else {
      performDelete(credentialId);
    }
  }

  function performDelete(credentialId: string) {
    passkeyStore.deleteCredential(credentialId);
    deleteConfirmId = null;
    onDelete?.(credentialId);
  }

  function cancelDelete() {
    deleteConfirmId = null;
  }

  function toggleExpand(credentialId: string) {
    expandedId = expandedId === credentialId ? null : credentialId;
  }

  async function copyToClipboard(text: string, id: string) {
    await navigator.clipboard.writeText(text);
    copiedId = id;
    setTimeout(() => (copiedId = null), 2000);
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp).toLocaleDateString("de-DE", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function formatAlgorithm(alg: number): string {
    switch (alg) {
      case COSE_ALGORITHMS.Ed25519:
        return "Ed25519";
      case COSE_ALGORITHMS.ES256:
        return "ES256 (P-256)";
      case COSE_ALGORITHMS.RS256:
        return "RS256";
      default:
        return `Unknown (${alg})`;
    }
  }

  function formatTransports(transports: string[] | undefined): string {
    if (!transports || transports.length === 0) return "Unbekannt";

    const labels: Record<string, string> = {
      internal: "Intern (TouchID/FaceID)",
      usb: "USB",
      nfc: "NFC",
      ble: "Bluetooth",
      hybrid: "Hybrid",
    };

    return transports.map((t) => labels[t] || t).join(", ");
  }
</script>

<Card class={compact ? "p-4" : ""}>
  <CardHeader class="pb-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Key class="h-5 w-5 text-primary" />
        <CardTitle class="text-lg">Deine Passkeys</CardTitle>
      </div>
      <Badge variant="secondary">{$passkeyCredentials.length} registriert</Badge
      >
    </div>
  </CardHeader>
  <CardContent class="space-y-4">
    {#if $passkeyCredentials.length === 0}
      <div class="py-8 text-center text-muted-foreground">
        <p>Noch keine Passkeys registriert.</p>
      </div>
    {:else}
      <div class="space-y-3">
        {#each $passkeyCredentials as credential (credential.id)}
          <div
            class="rounded-lg border transition-colors
              {credential.did === $activePasskeyDid
              ? 'border-primary bg-primary/5'
              : 'border-border bg-muted/30'}"
          >
            <!-- Header (immer sichtbar) -->
            <div class="flex items-center justify-between p-3">
              <div class="flex items-center gap-3">
                <div
                  class="flex h-10 w-10 items-center justify-center rounded-md bg-background"
                >
                  {#if credential.did === $activePasskeyDid}
                    <Check class="h-5 w-5 text-green-500" />
                  {:else}
                    <Key class="h-5 w-5 text-muted-foreground" />
                  {/if}
                </div>
                <div class="flex flex-col gap-0.5">
                  <span class="flex flex-wrap items-center gap-2 font-medium">
                    {credential.displayName || "Erynoa Passkey"}
                    {#if credential.isPrimary}
                      <Badge variant="default" class="text-xs">Primär</Badge>
                    {/if}
                    {#if credential.did === $activePasskeyDid}
                      <Badge
                        variant="secondary"
                        class="bg-green-500/10 text-green-500 text-xs"
                        >Aktiv</Badge
                      >
                    {/if}
                  </span>
                  <code class="font-mono text-xs text-muted-foreground">
                    {formatDidShort(credential.did, 32)}
                  </code>
                </div>
              </div>

              <Button
                variant="ghost"
                size="icon-sm"
                onclick={() => toggleExpand(credential.id)}
                title={expandedId === credential.id ? "Zuklappen" : "Details"}
              >
                {#if expandedId === credential.id}
                  <ChevronUp class="h-4 w-4" />
                {:else}
                  <ChevronDown class="h-4 w-4" />
                {/if}
              </Button>
            </div>

            <!-- Details (aufklappbar) -->
            {#if expandedId === credential.id}
              <div class="border-t border-border bg-background/50 p-4">
                <div
                  class="mb-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3"
                >
                  <div class="space-y-1">
                    <span
                      class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                      >DID</span
                    >
                    <div class="flex items-center gap-2">
                      <code
                        class="flex-1 break-all font-mono text-xs text-foreground"
                      >
                        {credential.did}
                      </code>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        onclick={() =>
                          copyToClipboard(
                            credential.did,
                            "did-" + credential.id,
                          )}
                        title="Kopieren"
                      >
                        {#if copiedId === "did-" + credential.id}
                          <Check class="h-3 w-3 text-green-500" />
                        {:else}
                          <Copy class="h-3 w-3" />
                        {/if}
                      </Button>
                    </div>
                  </div>

                  <div class="space-y-1">
                    <span
                      class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                      >Algorithmus</span
                    >
                    <p class="text-sm">
                      {formatAlgorithm(credential.algorithm)}
                    </p>
                  </div>

                  <div class="space-y-1">
                    <span
                      class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                      >Namespace</span
                    >
                    <p class="text-sm">{credential.namespace}</p>
                  </div>

                  <div class="space-y-1">
                    <span
                      class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                      >Erstellt</span
                    >
                    <p class="text-sm">{formatDate(credential.createdAt)}</p>
                  </div>

                  {#if credential.lastUsedAt}
                    <div class="space-y-1">
                      <span
                        class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                        >Zuletzt verwendet</span
                      >
                      <p class="text-sm">{formatDate(credential.lastUsedAt)}</p>
                    </div>
                  {/if}

                  <div class="space-y-1">
                    <span
                      class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                      >Authenticator</span
                    >
                    <p class="text-sm">
                      {formatTransports(credential.transports)}
                    </p>
                  </div>

                  <div class="space-y-1">
                    <span
                      class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                      >Credential ID</span
                    >
                    <div class="flex items-center gap-2">
                      <code class="font-mono text-xs text-foreground">
                        {credential.id.substring(0, 32)}...
                      </code>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        onclick={() =>
                          copyToClipboard(
                            credential.id,
                            "cred-" + credential.id,
                          )}
                        title="Kopieren"
                      >
                        {#if copiedId === "cred-" + credential.id}
                          <Check class="h-3 w-3 text-green-500" />
                        {:else}
                          <Copy class="h-3 w-3" />
                        {/if}
                      </Button>
                    </div>
                  </div>
                </div>

                <Separator class="my-4" />

                <!-- Detail-Actions -->
                <div class="flex flex-wrap items-center gap-3">
                  {#if credential.did !== $activePasskeyDid}
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={() => handleActivate(credential.did)}
                    >
                      <Check class="mr-2 h-4 w-4" />
                      Als aktiv setzen
                    </Button>
                  {/if}

                  {#if deleteConfirmId === credential.id}
                    <div
                      class="flex items-center gap-2 rounded-md bg-destructive/10 px-3 py-2"
                    >
                      <span class="text-sm text-destructive"
                        >Wirklich löschen?</span
                      >
                      <Button
                        variant="destructive"
                        size="sm"
                        onclick={() => performDelete(credential.id)}
                      >
                        Ja, löschen
                      </Button>
                      <Button variant="ghost" size="sm" onclick={cancelDelete}>
                        <X class="h-4 w-4" />
                      </Button>
                    </div>
                  {:else}
                    <Button
                      variant="ghost"
                      size="sm"
                      class="text-destructive hover:bg-destructive/10 hover:text-destructive"
                      onclick={() => initiateDelete(credential.id)}
                    >
                      <Trash2 class="mr-2 h-4 w-4" />
                      Löschen
                    </Button>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <Separator />

    <!-- Danger Zone -->
    <Alert variant="destructive" class="border-destructive/30 bg-destructive/5">
      <AlertTriangle class="h-4 w-4" />
      <AlertTitle>Gefahrenzone</AlertTitle>
      <AlertDescription class="space-y-3">
        <p>
          Alle Passkeys löschen entfernt alle lokalen Identitäten
          unwiderruflich.
        </p>
        <Button
          variant="destructive"
          size="sm"
          onclick={() => {
            if (
              confirm(
                "Wirklich ALLE Passkeys löschen? Dies kann nicht rückgängig gemacht werden.",
              )
            ) {
              passkeyStore.clearAll();
            }
          }}
        >
          <Trash2 class="mr-2 h-4 w-4" />
          Alle Passkeys löschen
        </Button>
      </AlertDescription>
    </Alert>
  </CardContent>
</Card>
