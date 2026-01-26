<script lang="ts">
	import type { HTMLLiAttributes } from 'svelte/elements';
	import type { Snippet } from 'svelte';

	interface Props extends HTMLLiAttributes {
		class?: string;
		children?: Snippet;
	}

	let { children, ...restProps }: Props = $props();
</script>

<li {...restProps}>{@render children?.()}</li>
