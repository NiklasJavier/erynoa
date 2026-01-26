<script lang="ts">
	import PageContent from '$lib/components/PageContent.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { user } from '$lib/auth';
	import { getAppConfig } from '$lib/config';
	import { onMount } from 'svelte';
	import { 
		Settings, 
		User, 
		Bell, 
		Shield, 
		Palette, 
		Database,
		Server,
		CheckCircle,
		ExternalLink
	} from 'lucide-svelte';

	let config = $state<ReturnType<typeof getAppConfig> | null>(null);

	onMount(() => {
		config = getAppConfig();
	});
</script>

<PageContent>
	<div class="grid gap-6 md:grid-cols-2">
		<!-- Profile Settings -->
		<Card.Root>
			<Card.Header>
				<Card.Title class="flex items-center gap-2">
					<User class="h-5 w-5" />
					Profile
				</Card.Title>
				<Card.Description>
					Your personal account information
				</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="space-y-2">
					<span class="text-sm font-medium">Username</span>
					<div class="p-2 bg-muted rounded-md text-sm">
						{$user?.profile?.preferred_username || '-'}
					</div>
				</div>
				<div class="space-y-2">
					<span class="text-sm font-medium">Email</span>
					<div class="p-2 bg-muted rounded-md text-sm">
						{$user?.profile?.email || '-'}
					</div>
				</div>
				<div class="space-y-2">
					<span class="text-sm font-medium">Name</span>
					<div class="p-2 bg-muted rounded-md text-sm">
						{$user?.profile?.name || '-'}
					</div>
				</div>
			</Card.Content>
			<Card.Footer>
				<a 
					href="http://localhost:8080/ui/console/users/me" 
					target="_blank"
					class="text-sm text-primary hover:underline flex items-center gap-1"
				>
					Edit in ZITADEL <ExternalLink class="h-3 w-3" />
				</a>
			</Card.Footer>
		</Card.Root>

		<!-- System Info -->
		<Card.Root>
			<Card.Header>
				<Card.Title class="flex items-center gap-2">
					<Server class="h-5 w-5" />
					System Information
				</Card.Title>
				<Card.Description>
					Application configuration and status
				</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="flex items-center justify-between">
					<span class="text-sm">Environment</span>
					<Badge variant="outline">{config?.environment || '-'}</Badge>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm">Version</span>
					<Badge variant="outline">v{config?.version || '-'}</Badge>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm">API URL</span>
					<span class="text-sm text-muted-foreground">{config?.urls.api || '-'}</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-sm">Auth Provider</span>
					<span class="text-sm text-muted-foreground">ZITADEL</span>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Security Settings -->
		<Card.Root>
			<Card.Header>
				<Card.Title class="flex items-center gap-2">
					<Shield class="h-5 w-5" />
					Security
				</Card.Title>
				<Card.Description>
					Authentication and security settings
				</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium">Two-Factor Authentication</p>
						<p class="text-xs text-muted-foreground">Additional security for your account</p>
					</div>
					<Badge variant="secondary">ZITADEL</Badge>
				</div>
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium">Active Sessions</p>
						<p class="text-xs text-muted-foreground">Manage your active sessions</p>
					</div>
					<Badge variant="secondary">ZITADEL</Badge>
				</div>
			</Card.Content>
			<Card.Footer>
				<a 
					href="http://localhost:8080/ui/console/users/me/security" 
					target="_blank"
					class="text-sm text-primary hover:underline flex items-center gap-1"
				>
					Manage Security <ExternalLink class="h-3 w-3" />
				</a>
			</Card.Footer>
		</Card.Root>

		<!-- Notifications -->
		<Card.Root>
			<Card.Header>
				<Card.Title class="flex items-center gap-2">
					<Bell class="h-5 w-5" />
					Notifications
				</Card.Title>
				<Card.Description>
					Configure notification preferences
				</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="flex items-center justify-center py-4 text-center">
					<div class="space-y-2">
						<p class="text-sm text-muted-foreground">
							Notification settings coming soon.
						</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>
	</div>
</PageContent>