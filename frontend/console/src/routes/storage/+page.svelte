<script lang="ts">
  /**
   * Storage Page
   *
   * TODO: Re-implement with ECLVM storage backend and Passkey-based DID authentication
   * The previous implementation used MinIO and OIDC tokens, which are no longer available.
   */
  import PageContent from "$lib/components/PageContent.svelte";
  import { Badge } from "@erynoa/ui/components/badge";
  import * as Card from "@erynoa/ui/components/card";
  import { Construction, HardDrive } from "lucide-svelte";
</script>

<PageContent>
  <Card.Root class="max-w-2xl mx-auto mt-12">
    <Card.Header class="text-center">
      <div
        class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-muted"
      >
        <HardDrive class="h-8 w-8 text-muted-foreground" />
      </div>
      <Card.Title class="text-2xl">Dezentraler Speicher</Card.Title>
      <Card.Description class="text-lg">Coming Soon</Card.Description>
    </Card.Header>
    <Card.Content class="text-center space-y-4">
      <Badge variant="outline" class="gap-2">
        <Construction class="h-3 w-3" />
        In Entwicklung
      </Badge>
      <p class="text-muted-foreground">
        Der dezentrale Speicher wird mit ECLVM-Integration und DID-basierter
        Zugriffskontrolle implementiert. Diese Funktion ist derzeit in aktiver
        Entwicklung.
      </p>
      <div class="text-sm text-muted-foreground/70 pt-4 border-t space-y-1">
        <p>Geplante Features:</p>
        <ul class="list-disc list-inside text-left max-w-md mx-auto">
          <li>Verschlüsselte Dateispeicherung</li>
          <li>DID-basierte Zugriffskontrolle</li>
          <li>Peer-to-Peer Synchronisation</li>
          <li>Versionierung und History</li>
        </ul>
      </div>
    </Card.Content>
  </Card.Root>
</PageContent>
