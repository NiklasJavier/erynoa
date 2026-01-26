<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		children?: import('svelte').Snippet;
	}

	let { class: className, children }: Props = $props();
</script>

<ul data-sidebar="menu" class={cn("flex w-full min-w-0 flex-col gap-1", className)}>
	{@render children?.()}
</ul>
