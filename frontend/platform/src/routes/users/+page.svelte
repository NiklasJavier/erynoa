<script lang="ts">
import PageContent from '$lib/components/PageContent.svelte'
import { Badge } from '$lib/components/ui/badge'
import * as Card from '$lib/components/ui/card'
import { Mail, Shield, UserPlus, Users } from 'lucide-svelte'
</script>

<PageContent>
	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2">
				<Users class="h-5 w-5" />
				User Management
			</Card.Title>
			<Card.Description>
				User management is handled through ZITADEL.
			</Card.Description>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="flex items-center justify-center py-8 text-center">
				<div class="space-y-4">
					<div class="flex h-20 w-20 mx-auto items-center justify-center rounded-full bg-muted">
						<Shield class="h-10 w-10 text-muted-foreground" />
					</div>
					<div class="space-y-2">
						<h3 class="text-lg font-medium">ZITADEL User Management</h3>
						<p class="text-sm text-muted-foreground max-w-md">
							Users are managed through the ZITADEL identity platform. 
							Visit the ZITADEL console to add, remove, or modify users.
						</p>
					</div>
					<a 
						href="http://localhost:8080/ui/console" 
						target="_blank"
						class="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
					>
						Open ZITADEL Console
					</a>
				</div>
			</div>
		</Card.Content>
	</Card.Root>

	<div class="grid gap-4 md:grid-cols-3 mt-6">
		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">Total Users</Card.Title>
				<Users class="h-4 w-4 text-muted-foreground" />
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">-</div>
				<p class="text-xs text-muted-foreground">Managed via ZITADEL</p>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">Active Sessions</Card.Title>
				<Shield class="h-4 w-4 text-muted-foreground" />
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">-</div>
				<p class="text-xs text-muted-foreground">Current active users</p>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">Pending Invites</Card.Title>
				<Mail class="h-4 w-4 text-muted-foreground" />
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">-</div>
				<p class="text-xs text-muted-foreground">Awaiting confirmation</p>
			</Card.Content>
		</Card.Root>
	</div>
</PageContent>