import type { ComponentType } from 'svelte';
import LayoutDashboard from 'lucide-svelte/icons/layout-dashboard';
import Users from 'lucide-svelte/icons/users';
import Settings from 'lucide-svelte/icons/settings';
import Database from 'lucide-svelte/icons/database';
import ChartBar from 'lucide-svelte/icons/chart-bar';
import FolderOpen from 'lucide-svelte/icons/folder-open';
import HelpCircle from 'lucide-svelte/icons/help-circle';
import Cog from 'lucide-svelte/icons/cog';
import Shield from 'lucide-svelte/icons/shield';
import CreditCard from 'lucide-svelte/icons/credit-card';
import User from 'lucide-svelte/icons/user';
import Bell from 'lucide-svelte/icons/bell';
import FileText from 'lucide-svelte/icons/file-text';
import BookOpen from 'lucide-svelte/icons/book-open';
import Code from 'lucide-svelte/icons/code';
import Rocket from 'lucide-svelte/icons/rocket';
import Blocks from 'lucide-svelte/icons/blocks';
import Server from 'lucide-svelte/icons/server';
import Activity from 'lucide-svelte/icons/activity';
import Gauge from 'lucide-svelte/icons/gauge';
import Key from 'lucide-svelte/icons/key';
import Lock from 'lucide-svelte/icons/lock';
import Globe from 'lucide-svelte/icons/globe';
// Godstack-spezifische Icons
import Container from 'lucide-svelte/icons/container';
import HardDrive from 'lucide-svelte/icons/hard-drive';
import Cpu from 'lucide-svelte/icons/cpu';
import Fingerprint from 'lucide-svelte/icons/fingerprint';
import KeyRound from 'lucide-svelte/icons/key-round';
import Network from 'lucide-svelte/icons/network';
import Box from 'lucide-svelte/icons/box';
import Plug from 'lucide-svelte/icons/plug';
import Terminal from 'lucide-svelte/icons/terminal';
import Workflow from 'lucide-svelte/icons/workflow';
import ScrollText from 'lucide-svelte/icons/scroll-text';
import Zap from 'lucide-svelte/icons/zap';
import GitBranch from 'lucide-svelte/icons/git-branch';
import Package from 'lucide-svelte/icons/package';
import CircleDot from 'lucide-svelte/icons/circle-dot';

// ============================================================================
// TYPES
// ============================================================================

/** Rollen für Route Guards */
export type UserRole = 'user' | 'admin' | 'editor';

/** Basis-Properties für alle Nav-Items */
interface NavItemBase {
	title: string;
	icon: ComponentType;
	badge?: string | number;
	disabled?: boolean;
	/** #3: Erforderliche Rollen (OR-Verknüpfung) */
	requiredRoles?: UserRole[];
	/** #3: Versteckt wenn keine Berechtigung (sonst disabled) */
	hideIfUnauthorized?: boolean;
}

/** Ein einfaches Navigationselement (Leaf Node) */
export interface NavItem extends NavItemBase {
	url: string;
}

/** Ein Navigationselement mit Sub-Items (rekursiv) */
export interface NavItemWithChildren extends NavItemBase {
	url?: string; // Optional - kann auch nur Container sein
	children: NavEntry[];
	defaultOpen?: boolean;
}

/** Union Type für alle Nav-Item Varianten (rekursiv) */
export type NavEntry = NavItem | NavItemWithChildren;

/** Prüft ob Entry Kinder hat */
export function hasChildren(entry: NavEntry): entry is NavItemWithChildren {
	return 'children' in entry && Array.isArray(entry.children) && entry.children.length > 0;
}

/** Gibt die URL zurück (oder undefined für Container-Items) */
export function getEntryUrl(entry: NavEntry): string | undefined {
	if (hasChildren(entry)) {
		return entry.url;
	}
	return entry.url;
}

export interface NavGroup {
	label: string;
	items: NavEntry[];
	collapsible?: boolean;
	defaultOpen?: boolean;
}

export interface NavConfig {
	brand: {
		name: string;
		subtitle: string;
		logo?: ComponentType;
		url: string;
	};
	topItems: NavEntry[]; // Eigenständige Items über den Gruppen
	groups: NavGroup[];
	footer: NavEntry[];
}

// ============================================================================
// NAVIGATION CONFIGURATION
// ============================================================================

export const navigationConfig: NavConfig = {
	brand: {
		name: 'Godstack',
		subtitle: 'Control Panel',
		url: '/',
	},
	// Eigenständige Items (über den Gruppen)
	topItems: [
		{ title: 'Dashboard', url: '/', icon: LayoutDashboard },
	],
	groups: [
		// ========================================
		// 1. PLATFORM - Anwendungs-Features
		// ========================================
		{
			label: 'Platform',
			items: [
				{ title: 'Analytics', url: '/analytics', icon: ChartBar },
				{
					title: 'Users',
					icon: Users,
					children: [
						{ title: 'Übersicht', url: '/users', icon: Users },
						{ title: 'Rollen', url: '/users/roles', icon: Shield },
						{ title: 'Berechtigungen', url: '/users/permissions', icon: Key },
					],
				},
			],
		},
		// ========================================
		// 2. DOCS - Dokumentations-Management
		// ========================================
		{
			label: 'Docs',
			items: [
				{
					title: 'Artikel',
					icon: FileText,
					children: [
						{ title: 'Alle', url: '/docs/articles', icon: FileText },
						{ title: 'Entwürfe', url: '/docs/articles/drafts', icon: ScrollText },
						{ title: 'Veröffentlicht', url: '/docs/articles/published', icon: Globe },
					],
				},
				{
					title: 'Kategorien',
					icon: Blocks,
					children: [
						{ title: 'Guides', url: '/docs/categories/guides', icon: BookOpen },
						{ title: 'API', url: '/docs/categories/api', icon: Code },
						{ title: 'Changelog', url: '/docs/categories/changelog', icon: GitBranch },
					],
				},
				{
					title: 'Medien',
					icon: FolderOpen,
					children: [
						{ title: 'Bilder', url: '/docs/media/images', icon: FolderOpen },
						{ title: 'Dateien', url: '/docs/media/files', icon: FileText },
					],
				},
			],
		},
		// ========================================
		// 3. INFRA - Infrastruktur & Konfiguration
		// ========================================
		{
			label: 'Infrastruktur',
			items: [
				// --- App-Konfiguration ---
				{
					title: 'Einstellungen',
					icon: Settings,
					children: [
						{ title: 'Allgemein', url: '/settings', icon: Cog },
						{ title: 'Profil', url: '/settings/profile', icon: User },
						{ title: 'Benachrichtigungen', url: '/settings/notifications', icon: Bell },
					],
				},
				{
					title: 'Sicherheit',
					icon: Shield,
					children: [
						{ title: 'API Keys', url: '/infra/security/api-keys', icon: KeyRound },
						{ title: 'Sessions', url: '/infra/security/sessions', icon: Activity },
						{ title: 'Audit Log', url: '/infra/security/audit', icon: ScrollText },
					],
				},
				// --- System & Services ---
				{
					title: 'System',
					icon: Server,
					children: [
						{ title: 'Übersicht', url: '/infra/system', icon: Server },
						{ title: 'Health', url: '/infra/system/health', icon: Activity },
						{ title: 'Logs', url: '/infra/system/logs', icon: Terminal },
					],
				},
				{
					title: 'Storage',
					icon: HardDrive,
					children: [
						{ title: 'Buckets', url: '/infra/storage/buckets', icon: Box },
						{ title: 'Dateien', url: '/infra/storage/files', icon: FolderOpen },
					],
				},
				{
					title: 'Cache',
					icon: Zap,
					children: [
						{ title: 'Übersicht', url: '/infra/cache', icon: Zap },
						{ title: 'Keys', url: '/infra/cache/keys', icon: Key },
					],
				},
			],
		},
	],
	footer: [],
};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/**
 * Rekursive Suche nach NavItem anhand URL
 */
function findNavItemRecursive(entries: NavEntry[], url: string): NavEntry | undefined {
	for (const entry of entries) {
		// Prüfe das aktuelle Item
		if (hasChildren(entry)) {
			if (entry.url === url) return entry;
			// Rekursiv in Children suchen
			const found = findNavItemRecursive(entry.children, url);
			if (found) return found;
		} else if (entry.url === url) {
			return entry;
		}
	}
	return undefined;
}

/**
 * Findet die erste URL in einer Gruppe (für Breadcrumb-Link)
 */
function findFirstUrlInEntries(entries: NavEntry[]): string | null {
	for (const entry of entries) {
		if (hasChildren(entry)) {
			if (entry.url) return entry.url;
			const childUrl = findFirstUrlInEntries(entry.children);
			if (childUrl) return childUrl;
		} else if (entry.url) {
			return entry.url;
		}
	}
	return null;
}

function findFirstUrlInGroup(group: NavGroup): string | null {
	return findFirstUrlInEntries(group.items);
}

export function getNavItemByUrl(url: string): NavEntry | undefined {
	// Suche in allen Gruppen
	for (const group of navigationConfig.groups) {
		const found = findNavItemRecursive(group.items, url);
		if (found) return found;
	}
	// Suche im Footer
	return findNavItemRecursive(navigationConfig.footer, url);
}

export function getPageTitle(url: string): string {
	const item = getNavItemByUrl(url);
	return item?.title || 'Page';
}

// ============================================================================
// SIDEBAR WIDTH CALCULATION
// ============================================================================

/**
 * Berechnet die maximale Verschachtelungstiefe der Navigation
 */
function getMaxDepthRecursive(entries: NavEntry[], currentDepth: number = 0): number {
	let maxDepth = currentDepth;
	for (const entry of entries) {
		if (hasChildren(entry)) {
			const childDepth = getMaxDepthRecursive(entry.children, currentDepth + 1);
			maxDepth = Math.max(maxDepth, childDepth);
		}
	}
	return maxDepth;
}

/**
 * Berechnet die empfohlene Sidebar-Breite basierend auf Navigation-Tiefe
 * Basis: 12rem + 1.5rem pro Verschachtelungsebene
 */
export function getSidebarWidth(): string {
	let maxDepth = 0;
	for (const group of navigationConfig.groups) {
		const groupDepth = getMaxDepthRecursive(group.items);
		maxDepth = Math.max(maxDepth, groupDepth);
	}
	const footerDepth = getMaxDepthRecursive(navigationConfig.footer);
	maxDepth = Math.max(maxDepth, footerDepth);
	
	// Basis 12rem + 1.5rem pro Tiefe
	const baseWidth = 12;
	const depthWidth = 1.5;
	const width = baseWidth + (maxDepth * depthWidth);
	
	return `${width}rem`;
}

// ============================================================================
// #1: BREADCRUMB GENERATION
// ============================================================================

export interface BreadcrumbItem {
	title: string;
	url: string;
	icon?: ComponentType;
	isCurrentPage?: boolean;
}

/**
 * Findet den Pfad (Hierarchie) zu einem NavItem anhand seiner URL
 * Gibt Array von NavEntries zurück: [Parent, ..., Item]
 */
function findNavPathRecursive(entries: NavEntry[], targetUrl: string, path: NavEntry[] = []): NavEntry[] | null {
	for (const entry of entries) {
		if (hasChildren(entry)) {
			// Prüfe ob eines der Children die Ziel-URL hat
			const childPath = findNavPathRecursive(entry.children, targetUrl, [...path, entry]);
			if (childPath) return childPath;
		} else if (entry.url === targetUrl) {
			return [...path, entry];
		}
	}
	return null;
}

/**
 * Findet den vollständigen Navigations-Pfad zu einer URL
 * Inkludiert die Gruppe (Label) als erstes Element
 */
function getNavPath(url: string): { group: NavGroup | null; path: NavEntry[] } {
	// Suche in topItems (ohne Gruppe)
	const topPath = findNavPathRecursive(navigationConfig.topItems, url);
	if (topPath) return { group: null, path: topPath };
	
	// Suche in allen Gruppen
	for (const group of navigationConfig.groups) {
		const path = findNavPathRecursive(group.items, url);
		if (path) return { group, path };
	}
	// Suche im Footer
	const footerPath = findNavPathRecursive(navigationConfig.footer, url);
	if (footerPath) return { group: null, path: footerPath };
	
	return { group: null, path: [] };
}

/**
 * Generiert Breadcrumbs basierend auf URL-Pfad und Navigation-Config
 * z.B. /docs/getting-started -> [Docs, Guides, Getting Started]
 */
export function getBreadcrumbs(pathname: string): BreadcrumbItem[] {
	const breadcrumbs: BreadcrumbItem[] = [];
	
	// Keine Breadcrumbs auf Home
	if (pathname === '/') {
		return breadcrumbs;
	}
	
	// Versuche vollständigen Nav-Pfad zu finden
	const { group, path: navPath } = getNavPath(pathname);
	
	if (navPath.length > 0) {
		// Füge Gruppe als erstes Breadcrumb hinzu (falls vorhanden)
		if (group) {
			// Finde die erste URL in dieser Gruppe für den Link
			const firstUrl = findFirstUrlInGroup(group);
			breadcrumbs.push({
				title: group.label,
				url: firstUrl ?? '/',
				isCurrentPage: false,
			});
		}
		
		// Navigation-Hierarchie gefunden - nutze diese für Breadcrumbs
		navPath.forEach((entry, i) => {
			const isLast = i === navPath.length - 1;
			breadcrumbs.push({
				title: entry.title,
				url: getEntryUrl(entry) ?? pathname,
				icon: entry.icon,
				isCurrentPage: isLast,
			});
		});
	} else {
		// Fallback: URL-Segmente
		const segments = pathname.split('/').filter(Boolean);
		let currentPath = '';
		
		for (let i = 0; i < segments.length; i++) {
			currentPath += '/' + segments[i];
			const isLast = i === segments.length - 1;
			
			const navItem = getNavItemByUrl(currentPath);
			
			if (navItem) {
				breadcrumbs.push({
					title: navItem.title,
					url: getEntryUrl(navItem) ?? currentPath,
					icon: navItem.icon,
					isCurrentPage: isLast,
				});
			} else {
				breadcrumbs.push({
					title: segments[i].charAt(0).toUpperCase() + segments[i].slice(1),
					url: currentPath,
					isCurrentPage: isLast,
				});
			}
		}
	}
	
	return breadcrumbs;
}

// ============================================================================
// #3: ROUTE GUARDS / PERMISSIONS
// ============================================================================

/**
 * Prüft ob User die erforderlichen Rollen für ein NavItem hat
 */
export function hasRequiredRoles(item: NavEntry, userRoles: UserRole[]): boolean {
	const requiredRoles = item.requiredRoles;
	if (!requiredRoles || requiredRoles.length === 0) return true;
	
	// OR-Verknüpfung: User braucht mindestens eine der Rollen
	return requiredRoles.some(role => userRoles.includes(role));
}

/**
 * Rekursive Filterung von Nav-Entries basierend auf User-Rollen
 */
function filterEntriesRecursive(entries: NavEntry[], userRoles: UserRole[]): NavEntry[] {
	return entries
		.filter(item => {
			const hasAccess = hasRequiredRoles(item, userRoles);
			const hideIfUnauthorized = item.hideIfUnauthorized;
			return hasAccess || !hideIfUnauthorized;
		})
		.map(item => {
			if (hasChildren(item)) {
				return {
					...item,
					children: filterEntriesRecursive(item.children, userRoles),
				};
			}
			return item;
		})
		// Entferne leere Container (alle Kinder gefiltert)
		.filter(item => !hasChildren(item) || item.children.length > 0);
}

/**
 * Filtert Navigation basierend auf User-Rollen
 */
export function getFilteredNavigation(userRoles: UserRole[]): NavConfig {
	return {
		...navigationConfig,
		topItems: filterEntriesRecursive(navigationConfig.topItems, userRoles),
		groups: navigationConfig.groups.map(group => ({
			...group,
			items: filterEntriesRecursive(group.items, userRoles),
		})).filter(group => group.items.length > 0),
		footer: filterEntriesRecursive(navigationConfig.footer, userRoles),
	};
}
