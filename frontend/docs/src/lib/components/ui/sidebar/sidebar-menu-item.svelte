<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		children?: import('svelte').Snippet;
	}

	let { class: className, children }: Props = $props();
</script>

<li data-sidebar="menu-item" class={cn("group/menu-item relative", className)}>
	{@render children?.()}
</li>
