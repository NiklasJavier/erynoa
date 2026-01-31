<script lang="ts">
import { cn } from '../../utils.js'
import type { Snippet } from 'svelte'
import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements'

type BaseProps = {
	class?: string
	isActive?: boolean
	size?: 'sm' | 'md'
	children?: Snippet
}

type AnchorProps = BaseProps & HTMLAnchorAttributes & { href: string }
type ButtonProps = BaseProps & HTMLButtonAttributes & { href?: never }
type Props = AnchorProps | ButtonProps

const {
	class: className,
	isActive = false,
	size = 'md',
	href,
	children,
	...restProps
}: Props = $props()

// Computed baseClasses als $derived
const baseClasses = $derived(
	cn(
		'flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-sidebar-foreground outline-none ring-sidebar-ring',
		'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
		'focus-visible:ring-2',
		'active:bg-sidebar-accent active:text-sidebar-accent-foreground',
		'disabled:pointer-events-none disabled:opacity-50',
		'aria-disabled:pointer-events-none aria-disabled:opacity-50',
		'[&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 [&>svg]:text-sidebar-accent-foreground',
		isActive && 'bg-sidebar-accent text-sidebar-accent-foreground font-medium',
		size === 'sm' && 'text-xs',
		size === 'md' && 'text-sm',
		className
	)
)
</script>

{#if href}
	<a {href} class={baseClasses} data-active={isActive} {...restProps as HTMLAnchorAttributes}>
		{@render children?.()}
	</a>
{:else}
	<button class={baseClasses} data-active={isActive} {...restProps as HTMLButtonAttributes}>
		{@render children?.()}
	</button>
{/if}
