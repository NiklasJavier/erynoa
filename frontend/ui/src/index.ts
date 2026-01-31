// ═══════════════════════════════════════════════════════════════════════════
// @erynoa/ui - Shared UI Library
// ═══════════════════════════════════════════════════════════════════════════
// Gemeinsame UI-Komponenten für alle Erynoa Frontends (console, docs, platform)

// Utilities
export {
	cn,
	type WithElementRef,
	type WithoutChild,
	type WithoutChildren,
	type WithoutChildrenOrChild,
} from './utils.js'

// ═══════════════════════════════════════════════════════════════════════════
// UI Components (shadcn/ui style)
// ═══════════════════════════════════════════════════════════════════════════

// Button
export {
	Button,
	buttonVariants,
	type ButtonProps,
	type ButtonSize,
	type ButtonVariant,
} from './components/button/index.js'

// Card
export {
	Card,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from './components/card/index.js'

// Input
export { Input } from './components/input/index.js'

// Label
export { Label } from './components/label/index.js'

// Badge
export {
	Badge,
	badgeVariants,
	type BadgeVariant,
} from './components/badge/index.js'

// Avatar
export { Avatar, AvatarFallback, AvatarImage } from './components/avatar/index.js'

// Alert
export { Alert, AlertDescription, AlertTitle } from './components/alert/index.js'

// Separator
export { Separator } from './components/separator/index.js'

// Checkbox
export { Checkbox } from './components/checkbox/index.js'

// Select
export * as Select from './components/select/index.js'

// Dialog
export * as Dialog from './components/dialog/index.js'

// Dropdown Menu
export * as DropdownMenu from './components/dropdown-menu/index.js'

// Sheet
export * as Sheet from './components/sheet/index.js'

// Tabs
export * as Tabs from './components/tabs/index.js'

// Table
export * as Table from './components/table/index.js'

// Collapsible
export * as Collapsible from './components/collapsible/index.js'

// Sidebar
export * as Sidebar from './components/sidebar/index.js'

// Sonner (Toast)
export { Toaster } from './components/sonner/index.js'

// ═══════════════════════════════════════════════════════════════════════════
// Layout Components
// ═══════════════════════════════════════════════════════════════════════════

export { default as Breadcrumbs } from './components/Breadcrumbs.svelte'
export { default as PageContent } from './components/PageContent.svelte'
export { default as ThemeToggle } from './components/ThemeToggle.svelte'
