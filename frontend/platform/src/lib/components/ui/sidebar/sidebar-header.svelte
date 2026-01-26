<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		children?: import('svelte').Snippet;
	}

	let { class: className, children }: Props = $props();
</script>

<div data-sidebar="header" class={cn("flex flex-col gap-2 p-2", className)}>
	{@render children?.()}
</div>
