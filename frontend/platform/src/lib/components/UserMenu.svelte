<script lang="ts">
import { authStore, isAuthenticated, user } from '$lib/auth'
import * as Avatar from '$lib/components/ui/avatar'
import { Button } from '$lib/components/ui/button'
import * as DropdownMenu from '$lib/components/ui/dropdown-menu'
import { LogOut, Settings, User } from 'lucide-svelte'

async function handleLogin() {
	await authStore.login()
}

async function handleLogout() {
	await authStore.logout()
}

// Get initials from user name
function getInitials(name: string | undefined): string {
	if (!name) return '?'
	return name
		.split(' ')
		.map((n) => n[0])
		.join('')
		.toUpperCase()
		.slice(0, 2)
}
</script>

{#if $isAuthenticated && $user}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      <Button variant="ghost" class="relative h-8 w-8 rounded-full">
        <Avatar.Root class="h-8 w-8">
          <Avatar.Image src={$user.profile?.picture} alt={$user.profile?.name} />
          <Avatar.Fallback>{getInitials($user.profile?.name)}</Avatar.Fallback>
        </Avatar.Root>
      </Button>
    </DropdownMenu.Trigger>
    <DropdownMenu.Content class="w-56" align="end">
      <DropdownMenu.Label class="font-normal">
        <div class="flex flex-col space-y-1">
          <p class="text-sm font-medium leading-none">{$user.profile?.name || 'User'}</p>
          <p class="text-xs leading-none text-muted-foreground">{$user.profile?.email}</p>
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
  <Button onclick={handleLogin} variant="default">
    Login
  </Button>
{/if}
