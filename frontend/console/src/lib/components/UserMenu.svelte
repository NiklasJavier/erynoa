<script lang="ts">
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import {
    activePasskeyCredential,
    activePasskeyDid,
    isPasskeyAuthenticated,
    passkeyStore,
  } from "$lib/auth/passkey";
  import * as Avatar from "@erynoa/ui/components/avatar";
  import { Button } from "@erynoa/ui/components/button";
  import * as DropdownMenu from "@erynoa/ui/components/dropdown-menu";
  import { LogOut, Settings, User } from "lucide-svelte";

  async function handleLogin() {
    await goto(`${base}/onboarding`);
  }

  async function handleLogout() {
    passkeyStore.clearActiveDid();
    await goto(`${base}/onboarding`);
  }

  // Get initials from DID
  function getInitials(did: string | null): string {
    if (!did) return "?";
    // Extract the last 4 chars of the DID for initials
    const suffix = did.split(":").pop() || "";
    return suffix.slice(0, 2).toUpperCase() || "?";
  }

  // Get display name from credential or DID
  const displayName = $derived(
    $activePasskeyCredential?.displayName ||
      $activePasskeyDid?.slice(-8) ||
      "User",
  );
</script>

{#if $isPasskeyAuthenticated && $activePasskeyDid}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      <Button variant="ghost" class="relative h-8 w-8 rounded-full">
        <Avatar.Root class="h-8 w-8">
          <Avatar.Fallback>{getInitials($activePasskeyDid)}</Avatar.Fallback>
        </Avatar.Root>
      </Button>
    </DropdownMenu.Trigger>
    <DropdownMenu.Content class="w-56" align="end">
      <DropdownMenu.Label class="font-normal">
        <div class="flex flex-col space-y-1">
          <p class="text-sm font-medium leading-none">
            {displayName}
          </p>
          <p class="text-xs leading-none text-muted-foreground font-mono">
            {$activePasskeyDid.slice(0, 20)}...
          </p>
        </div>
      </DropdownMenu.Label>
      <DropdownMenu.Separator />
      <DropdownMenu.Item>
        <User class="mr-2 h-4 w-4" />
        <span>Profile</span>
      </DropdownMenu.Item>
      <DropdownMenu.Item>
        <Settings class="mr-2 h-4 w-4" />
        <span>Settings</span>
      </DropdownMenu.Item>
      <DropdownMenu.Separator />
      <DropdownMenu.Item onSelect={handleLogout}>
        <LogOut class="mr-2 h-4 w-4" />
        <span>Logout</span>
      </DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
{:else}
  <Button onclick={handleLogin} variant="default">Login</Button>
{/if}
