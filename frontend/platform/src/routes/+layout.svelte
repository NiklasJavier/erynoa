<script lang="ts">
	import './layout.css';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { page } from '$app/stores';
	import { authStore, isAuthenticated, isLoading, authError } from '$lib/auth';
	import DashboardLayout from '$lib/components/DashboardLayout.svelte';
	import { Toaster } from '$lib/components/ui/sonner';
	import { ModeWatcher } from 'mode-watcher';
	import { Loader2 } from 'lucide-svelte';

	let { children } = $props();

	// Check if we're on the callback page
	let isCallbackPage = $derived($page.url.pathname === '/callback');

	// Debug state visible in UI
	let debugInfo = $state('Waiting for mount...');

	// Track if we've already started redirect
	let redirecting = $state(false);

	// Initialize auth in onMount to ensure we're client-side
	onMount(() => {
		debugInfo = 'Mounted, initializing auth...';
		console.log('[Layout] onMount triggered, initializing auth...');

		authStore.init().then(() => {
			debugInfo = 'Auth init complete!';
			console.log('[Layout] Auth init complete');
		}).catch((err) => {
			debugInfo = `Auth init failed: ${err?.message || err}`;
			console.error('[Layout] Auth init failed:', err);
		});
	});

	// Auto-redirect to Zitadel when not authenticated
	$effect(() => {
		if (!$isLoading && !$isAuthenticated && !isCallbackPage && !redirecting && browser) {
			redirecting = true;
			debugInfo = 'Redirecting to Zitadel...';
			console.log('[Layout] Not authenticated, redirecting to Zitadel...');
			
			// Speichere aktuelle URL für Redirect nach Login
			const returnUrl = $page.url.pathname + $page.url.search;
			if (returnUrl !== '/') {
				sessionStorage.setItem('auth_return_url', returnUrl);
				console.log('[Layout] Saved return URL:', returnUrl);
			}
			
			authStore.login();
		}
	});
</script>

<svelte:head>
	<title>Godstack</title>
	<meta name="description" content="Godstack - Modern Web Application" />
</svelte:head>

<ModeWatcher defaultMode="light" />

{#if isCallbackPage}
	<!-- Callback page has its own layout -->
	{@render children()}
{:else if $isLoading}
	<!-- Loading State -->
	<div class="flex min-h-screen items-center justify-center bg-background">
		<div class="flex flex-col items-center gap-4">
			<Loader2 class="h-8 w-8 animate-spin text-primary" />
			<p class="text-sm text-muted-foreground">Loading...</p>
			<p class="text-xs text-muted-foreground/50">{debugInfo}</p>
			{#if $authError}
				<p class="text-xs text-destructive mt-2">Error: {$authError}</p>
			{/if}
		</div>
	</div>
{:else if $isAuthenticated}
	<!-- Authenticated: Dashboard Layout -->
	<DashboardLayout>
		{@render children()}
	</DashboardLayout>
{:else}
	<!-- Not Authenticated: Redirecting to Zitadel -->
	<div class="flex min-h-screen items-center justify-center bg-background">
		<div class="flex flex-col items-center gap-4">
			<Loader2 class="h-8 w-8 animate-spin text-primary" />
			<p class="text-sm text-muted-foreground">Redirecting to login...</p>
		</div>
	</div>
{/if}

<Toaster />

