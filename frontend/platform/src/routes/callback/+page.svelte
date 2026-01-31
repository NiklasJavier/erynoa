<script lang="ts">
import { goto } from '$app/navigation'
import { authStore } from '$lib/auth'
import { getUser } from '$lib/auth/oidc'
import { Button } from '@erynoa/ui/components/button'
import * as Card from '@erynoa/ui/components/card'
import { AlertCircle, Loader2 } from 'lucide-svelte'
import { onMount } from 'svelte'

let error = $state<string | null>(null)

onMount(async () => {
	try {
		console.log('[Callback] Starting callback processing...')
		const user = await authStore.handleCallback()
		console.log('[Callback] Callback processed successfully, user:', {
			username: user?.profile?.preferred_username,
			expired: user?.expired,
			hasAccessToken: !!user?.access_token,
		})

		// Warte länger, damit der State vollständig aktualisiert wird
		// und alle reactive updates durchgelaufen sind
		await new Promise((resolve) => setTimeout(resolve, 300))

		// Verifiziere nochmal, dass der User gesetzt ist
		const currentUser = await getUser()
		if (!currentUser || currentUser.expired) {
			throw new Error('User not properly authenticated after callback')
		}

		console.log('[Callback] User verified, redirecting...')

		// Hole gespeicherte Return URL oder nutze Dashboard als Default
		const returnUrl = sessionStorage.getItem('auth_return_url') || '/'
		sessionStorage.removeItem('auth_return_url')
		console.log('[Callback] Redirecting to:', returnUrl)

		// Verwende replace statt goto, um die Callback-URL aus der History zu entfernen
		goto(returnUrl, { replaceState: true })
	} catch (err) {
		console.error('[Callback] Error:', err)
		error = err instanceof Error ? err.message : 'Authentication failed'
	}
})
</script>

<!-- Callback page renders outside of normal layout -->
<div class="flex min-h-screen items-center justify-center bg-background p-4">
  {#if error}
    <Card.Root class="w-full max-w-md">
      <Card.Header class="text-center">
        <div class="flex justify-center mb-4">
          <div
            class="flex h-14 w-14 items-center justify-center rounded-full bg-destructive/10"
          >
            <AlertCircle class="h-8 w-8 text-destructive" />
          </div>
        </div>
        <Card.Title class="text-destructive">Authentication Error</Card.Title>
        <Card.Description>
          There was a problem completing your login.
        </Card.Description>
      </Card.Header>
      <Card.Content>
        <p class="text-sm text-muted-foreground text-center">{error}</p>
      </Card.Content>
      <Card.Footer class="flex justify-center">
        <Button href="/" variant="outline">Return to Login</Button>
      </Card.Footer>
    </Card.Root>
  {:else}
    <Card.Root class="w-full max-w-md">
      <Card.Content class="pt-6">
        <div class="text-center space-y-4">
          <Loader2 class="h-8 w-8 animate-spin mx-auto text-primary" />
          <div>
            <p class="font-medium">Completing authentication...</p>
            <p class="text-sm text-muted-foreground">
              Please wait while we verify your credentials.
            </p>
          </div>
        </div>
      </Card.Content>
    </Card.Root>
  {/if}
</div>
