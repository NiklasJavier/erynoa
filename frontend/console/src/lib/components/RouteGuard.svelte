<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { isAuthenticated, isLoading, user } from '$lib/auth';
	import { getPageMeta, type PageAuth } from '$lib/config';
	import type { UserRole } from '$lib/config';
	import type { Snippet } from 'svelte';
	import { Loader2, ShieldAlert } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	interface Props {
		/** Custom Auth-Config (überschreibt pageConfig) */
		auth?: PageAuth;
		/** Content wenn autorisiert */
		children: Snippet;
		/** Custom "Nicht autorisiert" Content */
		unauthorized?: Snippet;
	}

	let { auth, children, unauthorized }: Props = $props();

	// Hole Auth-Config aus pageConfig wenn nicht explizit übergeben
	const pageAuth = $derived(auth ?? getPageMeta($page.url.pathname).auth);

	// Extrahiere Rollen aus OIDC-Claims (anpassbar je nach Auth-Provider)
	const userRoles = $derived<UserRole[]>(() => {
		if (!$user?.profile) return ['user'];
		
		// ZITADEL: Rollen können in verschiedenen Claims sein
		const roles: UserRole[] = ['user']; // Default-Rolle
		
		// Check für Admin-Rolle (anpassen je nach ZITADEL-Config)
		const profile = $user.profile as Record<string, unknown>;
		if (profile['urn:zitadel:iam:org:project:roles']) {
			const zitadelRoles = profile['urn:zitadel:iam:org:project:roles'] as Record<string, unknown>;
			if ('admin' in zitadelRoles) roles.push('admin');
			if ('editor' in zitadelRoles) roles.push('editor');
		}
		
		return roles;
	});

	// Auth-Check
	const isAuthorized = $derived(() => {
		if (!pageAuth) return true; // Keine Auth-Anforderung = öffentlich
		
		// Check: Login erforderlich?
		if (pageAuth.required && !$isAuthenticated) return false;
		
		// Check: Rollen erforderlich?
		if (pageAuth.roles && pageAuth.roles.length > 0) {
			const hasRole = pageAuth.roles.some(role => userRoles().includes(role));
			if (!hasRole) return false;
		}
		
		return true;
	});

	// Redirect wenn nicht autorisiert
	$effect(() => {
		if ($isLoading) return; // Warte auf Auth-Init
		
		if (!isAuthorized()) {
			if (!$isAuthenticated && pageAuth?.required) {
				// Nicht eingeloggt -> Login
				// Auth-Redirect wird vom Auth-Store gehandled
				return;
			}
			
			if (pageAuth?.redirectTo) {
				goto(pageAuth.redirectTo);
			}
		}
	});
</script>

{#if $isLoading}
	<!-- Loading Auth State -->
	<div class="flex items-center justify-center min-h-[60vh]">
		<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
	</div>

{:else if !isAuthorized()}
	<!-- Nicht autorisiert -->
	{#if unauthorized}
		{@render unauthorized()}
	{:else}
		<div class="flex items-center justify-center min-h-[60vh] p-4">
			<Card.Root class="w-full max-w-md">
				<Card.Header class="text-center">
					<div class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
						<ShieldAlert class="h-6 w-6 text-destructive" />
					</div>
					<Card.Title>Access Denied</Card.Title>
					<Card.Description>
						{#if !$isAuthenticated}
							You need to be logged in to access this page.
						{:else}
							You don't have permission to access this page.
						{/if}
					</Card.Description>
				</Card.Header>
				<Card.Footer class="flex justify-center">
					<Button href="/" variant="outline">Go to Dashboard</Button>
				</Card.Footer>
			</Card.Root>
		</div>
	{/if}

{:else}
	<!-- Autorisiert - zeige Content -->
	{@render children()}
{/if}
