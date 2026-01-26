<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/auth';
	import { Loader2, AlertCircle } from 'lucide-svelte';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';

	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			await authStore.handleCallback();
			
			// Hole gespeicherte Return URL oder nutze Dashboard als Default
			const returnUrl = sessionStorage.getItem('auth_return_url') || '/';
			sessionStorage.removeItem('auth_return_url');
			console.log('[Callback] Redirecting to:', returnUrl);
			
			goto(returnUrl);
		} catch (err) {
			console.error('[Callback] Error:', err);
			error = err instanceof Error ? err.message : 'Authentication failed';
		}
	});
</script>

<!-- Callback page renders outside of normal layout -->
<div class="flex min-h-screen items-center justify-center bg-background p-4">
	{#if error}
		<Card.Root class="w-full max-w-md">
			<Card.Header class="text-center">
				<div class="flex justify-center mb-4">
					<div class="flex h-14 w-14 items-center justify-center rounded-full bg-destructive/10">
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
						<p class="text-sm text-muted-foreground">Please wait while we verify your credentials.</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
