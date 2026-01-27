<script lang="ts">
import { page } from '$app/stores'
import { authStore, user } from '$lib/auth'
import * as Avatar from '$lib/components/ui/avatar'
import { Badge } from '$lib/components/ui/badge'
import * as Collapsible from '$lib/components/ui/collapsible'
import * as DropdownMenu from '$lib/components/ui/dropdown-menu'
import * as Sidebar from '$lib/components/ui/sidebar'
import { getSidebarContext } from '$lib/components/ui/sidebar'
import {
	type NavEntry,
	type UserRole,
	getEntryUrl,
	getFilteredNavigation,
	hasChildren,
	navigationConfig,
} from '$lib/config'
import ChevronRight from 'lucide-svelte/icons/chevron-right'
import ChevronsUpDown from 'lucide-svelte/icons/chevrons-up-down'
import LogOut from 'lucide-svelte/icons/log-out'
import Sparkles from 'lucide-svelte/icons/sparkles'

interface Props {
	variant?: 'sidebar' | 'floating' | 'inset'
}

const { variant = 'inset' }: Props = $props()

// Sidebar Context für collapsed State
const { state: sidebarState } = getSidebarContext()
const isCollapsed = $derived($sidebarState === 'collapsed')

// Benutzerrollen aus OIDC-Claims extrahieren
const userRoles = $derived(() => {
	if (!$user?.profile) return ['user'] as UserRole[]

	const roles: UserRole[] = ['user']
	const profile = $user.profile as Record<string, unknown>

	// ZITADEL: Rollen aus project roles claim
	if (profile['urn:zitadel:iam:org:project:roles']) {
		const zitadelRoles = profile['urn:zitadel:iam:org:project:roles'] as Record<string, unknown>
		if ('admin' in zitadelRoles) roles.push('admin')
		if ('editor' in zitadelRoles) roles.push('editor')
	}

	return roles
})

// Gefilterte Navigation basierend auf Benutzerrollen
const filteredConfig = $derived(() => {
	const roles = userRoles()
	return getFilteredNavigation(roles)
})

// Navigation aus gefilterter Config
const { brand } = navigationConfig // Brand immer sichtbar
const topItems = $derived.by(() => filteredConfig().topItems)
const groups = $derived.by(() => filteredConfig().groups)
const footer = $derived.by(() => filteredConfig().footer)

// Exakter Match - für Leaf-Items (ohne Kinder)
function isExactActive(url: string | undefined): boolean {
	if (!url) return false
	return $page.url.pathname === url
}

// Prüft ob ein Item mit Children (oder deren Children) aktiv ist
function isChildActiveRecursive(entry: NavEntry): boolean {
	if (!hasChildren(entry)) {
		return isExactActive(entry.url)
	}
	return entry.children.some((child) => isChildActiveRecursive(child))
}
</script>

{#snippet renderDropdownItems(children: NavEntry[])}
	{#each children as child}
		{#if hasChildren(child)}
			<DropdownMenu.Sub>
				<DropdownMenu.SubTrigger>
					{@const Icon = child.icon}
					<Icon class="mr-2 size-4" />
					<span>{child.title}</span>
				</DropdownMenu.SubTrigger>
				<DropdownMenu.SubContent>
					{@render renderDropdownItems(child.children)}
				</DropdownMenu.SubContent>
			</DropdownMenu.Sub>
		{:else}
			<DropdownMenu.Item>
				{@const Icon = child.icon}
				<a href={child.url} class="flex items-center w-full">
					<Icon class="mr-2 size-4" />
					<span>{child.title}</span>
					{#if child.badge}
						<Badge variant="secondary" class="ml-auto text-xs">
							{child.badge}
						</Badge>
					{/if}
				</a>
			</DropdownMenu.Item>
		{/if}
	{/each}
{/snippet}

{#snippet renderNavEntry(entry: NavEntry, depth: number = 0, groupLabel: string = '')}
	{#if hasChildren(entry)}
		{#if isCollapsed && depth === 0}
			<!-- Collapsed Mode: DropdownMenu für Items mit Children -->
			<Sidebar.MenuItem>
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						<Sidebar.MenuButton isActive={isChildActiveRecursive(entry)}>
							{@const Icon = entry.icon}
							<Icon class="size-4" />
							<span>{entry.title}</span>
							<ChevronRight class="ml-auto size-4" />
						</Sidebar.MenuButton>
					</DropdownMenu.Trigger>
					<DropdownMenu.Content side="right" align="start" class="min-w-[200px]">
						<DropdownMenu.Label class="flex items-center justify-between">
							<span>{entry.title}</span>
							{#if groupLabel}
								<span class="text-xs font-normal text-muted-foreground">{groupLabel}</span>
							{/if}
						</DropdownMenu.Label>
						<DropdownMenu.Separator />
						{@render renderDropdownItems(entry.children)}
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</Sidebar.MenuItem>
		{:else}
			<!-- Expanded Mode: Collapsible mit Sub-Items (rekursiv) -->
			<Collapsible.Root open={entry.defaultOpen || isChildActiveRecursive(entry)} class="group/collapsible">
				<Sidebar.MenuItem>
					<Collapsible.Trigger>
						{#if depth === 0}
							<Sidebar.MenuButton>
								{@const Icon = entry.icon}
								<Icon class="size-4" />
								<span>{entry.title}</span>
								<ChevronRight class="ml-auto size-4 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
							</Sidebar.MenuButton>
						{:else}
							<Sidebar.MenuSubButton>
								<span>{entry.title}</span>
								<ChevronRight class="ml-auto size-4 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
							</Sidebar.MenuSubButton>
						{/if}
					</Collapsible.Trigger>
					<Collapsible.Content>
						<Sidebar.MenuSub>
							{#each entry.children as child}
								<Sidebar.MenuSubItem>
									{@render renderNavEntry(child, depth + 1)}
								</Sidebar.MenuSubItem>
							{/each}
						</Sidebar.MenuSub>
					</Collapsible.Content>
				</Sidebar.MenuItem>
			</Collapsible.Root>
		{/if}
	{:else}
		<!-- Leaf Node (einfaches Item) -->
		{#if depth === 0}
			<Sidebar.MenuItem>
				<Sidebar.MenuButton 
					href={entry.url} 
					isActive={isExactActive(entry.url)}
				>
					{@const Icon = entry.icon}
					<Icon class="size-4" />
					<span>{entry.title}</span>
					{#if entry.badge}
						<Badge variant="secondary" class="ml-auto text-xs">
							{entry.badge}
						</Badge>
					{/if}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		{:else}
			<Sidebar.MenuSubButton 
				href={entry.url} 
				isActive={isExactActive(entry.url)}
			>
				<span>{entry.title}</span>
				{#if entry.badge}
					<Badge variant="secondary" class="ml-auto text-xs">
						{entry.badge}
					</Badge>
				{/if}
			</Sidebar.MenuSubButton>
		{/if}
	{/if}
{/snippet}

<Sidebar.Root {variant} collapsible="icon">
	<!-- Brand Header - dynamisch aus Config -->
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg" href={brand.url}>
					<div class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
						{#if brand.logo}
							{@const Logo = brand.logo}
							<Logo class="size-4" />
						{:else}
							<Sparkles class="size-4" />
						{/if}
					</div>
					<div class="grid flex-1 text-left text-sm leading-tight">
						<span class="truncate font-semibold">{brand.name}</span>
						<span class="truncate text-xs text-muted-foreground">{brand.subtitle}</span>
					</div>
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	
	<Sidebar.Content>
		<!-- Top Items (eigenständig, ohne Gruppe) -->
		{#if topItems.length > 0}
			<Sidebar.Group>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each topItems as entry}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton 
									href={entry.url} 
									isActive={isExactActive(entry.url)}
									class="bg-sidebar-accent/50 border border-sidebar-border hover:bg-sidebar-accent font-medium"
								>
									{@const Icon = entry.icon}
									<Icon class="size-4" />
									<span>{entry.title}</span>
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/if}
		
		<!-- Dynamische Navigation Groups -->
		{#each groups as group, i}
			{#if i > 0 || topItems.length > 0}
				<!-- Separator zwischen Groups (nur sichtbar im collapsed mode) -->
				<div class="mx-2 my-1 hidden h-px bg-sidebar-border group-data-[collapsible=icon]:block"></div>
			{/if}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{group.label}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each group.items as entry}
							{@render renderNavEntry(entry, 0, group.label)}
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>
	
	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<DropdownMenu.Root>
					<DropdownMenu.Trigger class="w-full">
						<Sidebar.MenuButton size="lg" class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground">
							<Avatar.Root class="h-8 w-8 rounded-lg">
								<Avatar.Fallback class="rounded-lg">
									{$user?.profile?.preferred_username?.charAt(0).toUpperCase() || 'U'}
								</Avatar.Fallback>
							</Avatar.Root>
							<div class="grid flex-1 text-left text-sm leading-tight">
								<span class="truncate font-semibold">
									{$user?.profile?.preferred_username || 'User'}
								</span>
								<span class="truncate text-xs text-muted-foreground">
									{$user?.profile?.email || ''}
								</span>
							</div>
							<ChevronsUpDown class="ml-auto size-4" />
						</Sidebar.MenuButton>
					</DropdownMenu.Trigger>
					<DropdownMenu.Content
						class="w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg"
						side="bottom"
						align="end"
						sideOffset={4}
					>
						<DropdownMenu.Label class="p-0 font-normal">
							<div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
								<Avatar.Root class="h-8 w-8 rounded-lg">
									<Avatar.Fallback class="rounded-lg">
										{$user?.profile?.preferred_username?.charAt(0).toUpperCase() || 'U'}
									</Avatar.Fallback>
								</Avatar.Root>
								<div class="grid flex-1 text-left text-sm leading-tight">
									<span class="truncate font-semibold">{$user?.profile?.preferred_username || 'User'}</span>
									<span class="truncate text-xs text-muted-foreground">{$user?.profile?.email || ''}</span>
								</div>
							</div>
						</DropdownMenu.Label>
						<DropdownMenu.Separator />
						<DropdownMenu.Item onclick={() => authStore.logout()}>
							<LogOut class="mr-2 h-4 w-4" />
							Log out
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
</Sidebar.Root>
