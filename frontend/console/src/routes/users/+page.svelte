<script lang="ts">
  /**
   * Users Page
   *
   * TODO: Implement DID-based user/peer discovery
   * With Passkey-based DID authentication, users are self-sovereign.
   * This page will show known peers and contacts.
   */
  import PageContent from "$lib/components/PageContent.svelte";
  import { Badge } from "@erynoa/ui/components/badge";
  import * as Card from "@erynoa/ui/components/card";
  import { Construction, Fingerprint, UserPlus, Users } from "lucide-svelte";
</script>

<PageContent>
  <Card.Root class="max-w-2xl mx-auto mt-12">
    <Card.Header class="text-center">
      <div
        class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-muted"
      >
        <Users class="h-8 w-8 text-muted-foreground" />
      </div>
      <Card.Title class="text-2xl">Peers & Kontakte</Card.Title>
      <Card.Description class="text-lg">Coming Soon</Card.Description>
    </Card.Header>
    <Card.Content class="text-center space-y-4">
      <Badge variant="outline" class="gap-2">
        <Construction class="h-3 w-3" />
        In Entwicklung
      </Badge>
      <p class="text-muted-foreground">
        Mit der selbstbestimmten Identität (DID) verwaltest du deine Kontakte
        dezentral. Diese Funktion ist derzeit in aktiver Entwicklung.
      </p>
      <div class="text-sm text-muted-foreground/70 pt-4 border-t space-y-1">
        <p>Geplante Features:</p>
        <ul class="list-disc list-inside text-left max-w-md mx-auto">
          <li>Peer-Entdeckung im Netzwerk</li>
          <li>Kontaktanfragen via DID</li>
          <li>Vertrauensnetzwerk-Verwaltung</li>
          <li>Gruppenbildung und Gilden</li>
        </ul>
      </div>
    </Card.Content>
  </Card.Root>

  <div class="grid gap-4 md:grid-cols-3 mt-6">
    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Bekannte Peers</Card.Title>
        <Users class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">0</div>
        <p class="text-xs text-muted-foreground">Im lokalen Netzwerk</p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Verifizierte DIDs</Card.Title>
        <Fingerprint class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">0</div>
        <p class="text-xs text-muted-foreground">
          Vertrauenswürdige Identitäten
        </p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Offene Anfragen</Card.Title>
        <UserPlus class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">0</div>
        <p class="text-xs text-muted-foreground">Ausstehende Verbindungen</p>
      </Card.Content>
    </Card.Root>
  </div>
</PageContent>
