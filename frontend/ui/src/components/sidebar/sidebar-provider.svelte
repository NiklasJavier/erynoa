<script lang="ts" module>
import { getContext, setContext } from 'svelte'
import { type Writable, writable } from 'svelte/store'

const _SIDEBAR_COOKIE_NAME = 'sidebar:state'
const _SIDEBAR_COOKIE_MAX_AGE = 60 * 60 * 24 * 7
const _SIDEBAR_WIDTH = '16rem'
const _SIDEBAR_WIDTH_MOBILE = '18rem'
const _SIDEBAR_WIDTH_ICON = '3rem'
const _SIDEBAR_KEYBOARD_SHORTCUT = 'b'

export interface SidebarContext {
	state: Writable<'expanded' | 'collapsed'>
	open: Writable<boolean>
	setOpen: (value: boolean) => void
	openMobile: Writable<boolean>
	setOpenMobile: (value: boolean) => void
	isMobile: Writable<boolean>
	toggleSidebar: () => void
}

const SIDEBAR_CONTEXT_KEY = 'sidebar-context'

export function setSidebarContext(context: SidebarContext) {
	setContext(SIDEBAR_CONTEXT_KEY, context)
}

export function getSidebarContext(): SidebarContext {
	return getContext(SIDEBAR_CONTEXT_KEY)
}
</script>

<script lang="ts">
	import { onMount } from 'svelte';
	import { cn } from '../../utils.js';

	interface Props {
		defaultOpen?: boolean;
		open?: boolean;
		onOpenChange?: (open: boolean) => void;
		class?: string;
		style?: string;
		children?: import('svelte').Snippet;
	}

	let {
		defaultOpen = true,
		open: openProp = $bindable(defaultOpen),
		onOpenChange,
		class: className,
		style,
		children,
	}: Props = $props();

	const isMobile = writable(false);
	const openMobile = writable(false);
	const open = writable(openProp);
	const state = writable<'expanded' | 'collapsed'>(openProp ? 'expanded' : 'collapsed');

	$effect(() => {
		open.set(openProp);
		state.set(openProp ? 'expanded' : 'collapsed');
	});

	function setOpen(value: boolean) {
		open.set(value);
		state.set(value ? 'expanded' : 'collapsed');
		openProp = value;
		onOpenChange?.(value);
	}

	function setOpenMobile(value: boolean) {
		openMobile.set(value);
	}

	function toggleSidebar() {
		if ($isMobile) {
			openMobile.update(v => !v);
		} else {
			setOpen(!$open);
		}
	}

	setSidebarContext({
state,
open,
setOpen,
openMobile,
setOpenMobile,
isMobile,
toggleSidebar,
});

	onMount(() => {
		const checkMobile = () => {
			isMobile.set(window.innerWidth < 768);
		};
		checkMobile();
		window.addEventListener('resize', checkMobile);

		// Keyboard shortcut
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === _SIDEBAR_KEYBOARD_SHORTCUT && (e.metaKey || e.ctrlKey)) {
				e.preventDefault();
				toggleSidebar();
			}
		};
		window.addEventListener('keydown', handleKeyDown);

		return () => {
			window.removeEventListener('resize', checkMobile);
			window.removeEventListener('keydown', handleKeyDown);
		};
	});

	// Merge styles properly - custom style takes precedence (comes AFTER default)
	const defaultStyle = `--sidebar-width: ${_SIDEBAR_WIDTH}; --sidebar-width-icon: ${_SIDEBAR_WIDTH_ICON}; --sidebar-width-mobile: ${_SIDEBAR_WIDTH_MOBILE};`;
	const combinedStyle = $derived(style ? `${defaultStyle} ${style}` : defaultStyle);
</script>

<div
	style={combinedStyle}
	class={cn(
"group/sidebar-wrapper flex min-h-svh w-full has-data-[variant=inset]:bg-sidebar",
className
)}
>
	{@render children?.()}
</div>
