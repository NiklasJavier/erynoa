// ============================================================================
// PAGE CONFIGURATION
// ============================================================================

import type { UserRole } from './navigation'

/** Auth-Anforderungen für eine Seite */
export interface PageAuth {
	/** Erfordert eingeloggten User */
	required?: boolean
	/** Erforderliche Rollen (OR-Verknüpfung) */
	roles?: UserRole[]
	/** Redirect-URL wenn nicht autorisiert */
	redirectTo?: string
}

export interface PageMeta {
	title: string
	description?: string
	showHeader?: boolean // Default: true
	/** Browser Tab Titel Suffix (default: "| Godstack") */
	documentTitle?: string
	/** Breadcrumbs anzeigen (default: true außer auf /) */
	showBreadcrumbs?: boolean
	/** #3: Auth-Anforderungen */
	auth?: PageAuth
}

/** App Name für Document Title */
export const APP_NAME = 'Godstack'

export const pageConfig: Record<string, PageMeta> = {
	'/': {
		title: 'Dashboard',
		description: 'Overview of your application',
		showHeader: false,
		showBreadcrumbs: false,
		auth: { required: true },
	},
	'/analytics': {
		title: 'Analytics',
		description: 'View your analytics and metrics',
		showBreadcrumbs: true,
		auth: { required: true },
	},
	'/users': {
		title: 'Users',
		description: 'Manage users and their permissions',
		showBreadcrumbs: true,
		auth: { required: true },
	},
	'/storage': {
		title: 'Storage',
		description: 'Manage your files and buckets',
		showBreadcrumbs: true,
		auth: { required: true },
	},
	'/documents': {
		title: 'Documents',
		description: 'Manage your documents',
		showBreadcrumbs: true,
		auth: { required: true },
	},
	'/settings': {
		title: 'Settings',
		description: 'Manage your account and application settings',
		showBreadcrumbs: true,
		auth: { required: true },
	},
	'/help': {
		title: 'Help',
		description: 'Get help and support',
		showBreadcrumbs: true,
		auth: { required: true },
	},
	// Beispiel: Admin-only Seite
	// '/admin': {
	// 	title: 'Admin Panel',
	// 	description: 'System administration',
	// 	auth: { required: true, roles: ['admin'], redirectTo: '/' },
	// },
}

export function getPageMeta(path: string): PageMeta {
	return (
		pageConfig[path] || {
			title: 'Page',
			description: '',
		}
	)
}

/** Generiert den vollständigen Document Title */
export function getDocumentTitle(path: string): string {
	const meta = getPageMeta(path)
	if (meta.documentTitle) return meta.documentTitle
	if (path === '/') return APP_NAME
	return `${meta.title} | ${APP_NAME}`
}
