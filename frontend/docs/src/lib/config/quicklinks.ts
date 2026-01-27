import type { ComponentType } from 'svelte'
import {
	type NavEntry,
	type NavGroup,
	getEntryUrl,
	hasChildren,
	navigationConfig,
} from './navigation'

// ============================================================================
// QUICKLINKS CONFIGURATION
// ============================================================================

export interface QuickLink {
	title: string
	description?: string
	url: string
	icon?: ComponentType
	category?: string
}

export interface QuickLinkGroup {
	label: string
	description?: string
	links: QuickLink[]
}

/**
 * Konfiguration für Dashboard-Quicklinks
 * Verweist auf URLs aus der Navigation oder definiert eigene
 */
export interface DashboardQuickLinksConfig {
	/** Quicklinks aus der Navigation (per URL oder Pfad) */
	fromNavigation: string[]
	/** Zusätzliche manuelle Quicklinks */
	custom: QuickLink[]
}

export const dashboardQuickLinksConfig: DashboardQuickLinksConfig = {
	// URLs aus der Navigation, die als Quicklinks erscheinen sollen
	fromNavigation: [
		'/users',
		'/analytics',
		'/infra/storage/buckets',
		'/infra/system',
		'/docs/articles',
		'/settings',
	],
	// Zusätzliche manuelle Quicklinks
	custom: [],
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/**
 * Sucht rekursiv nach einem NavEntry anhand seiner URL
 */
function findNavEntryByUrl(
	entries: NavEntry[],
	url: string
): { entry: NavEntry; group?: NavGroup } | null {
	for (const entry of entries) {
		if (hasChildren(entry)) {
			if (entry.url === url) return { entry }
			const found = findNavEntryByUrl(entry.children, url)
			if (found) return found
		} else if (entry.url === url) {
			return { entry }
		}
	}
	return null
}

/**
 * Findet NavEntry und zugehörige Gruppe
 */
function findNavEntryWithGroup(url: string): { entry: NavEntry; group?: NavGroup } | null {
	// Suche in topItems
	for (const entry of navigationConfig.topItems) {
		if (hasChildren(entry)) {
			const found = findNavEntryByUrl(entry.children, url)
			if (found) return found
		} else if (entry.url === url) {
			return { entry }
		}
	}

	// Suche in groups
	for (const group of navigationConfig.groups) {
		const found = findNavEntryByUrl(group.items, url)
		if (found) return { ...found, group }
	}

	return null
}

/**
 * Konvertiert NavEntry zu QuickLink
 */
function navEntryToQuickLink(entry: NavEntry, group?: NavGroup): QuickLink {
	return {
		title: entry.title,
		url: getEntryUrl(entry) ?? '/',
		icon: entry.icon,
		category: group?.label,
	}
}

/**
 * Generiert Quicklinks aus der Konfiguration
 */
export function getDashboardQuickLinks(): QuickLink[] {
	const links: QuickLink[] = []

	// Links aus Navigation holen
	for (const url of dashboardQuickLinksConfig.fromNavigation) {
		const found = findNavEntryWithGroup(url)
		if (found) {
			links.push(navEntryToQuickLink(found.entry, found.group))
		}
	}

	// Custom Links hinzufügen
	links.push(...dashboardQuickLinksConfig.custom)

	return links
}

/**
 * Generiert gruppierte Quicklinks (nach Kategorie)
 */
export function getDashboardQuickLinksGrouped(): QuickLinkGroup[] {
	const links = getDashboardQuickLinks()
	const grouped = new Map<string, QuickLink[]>()

	for (const link of links) {
		const category = link.category ?? 'Allgemein'
		if (!grouped.has(category)) {
			grouped.set(category, [])
		}
		grouped.get(category)?.push(link)
	}

	return Array.from(grouped.entries()).map(([label, links]) => ({
		label,
		links,
	}))
}
