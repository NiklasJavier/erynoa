<script lang="ts">
import type { Snippet } from 'svelte'
import type { HTMLLiAttributes } from 'svelte/elements'

interface Props extends HTMLLiAttributes {
	class?: string
	children?: Snippet
}

const { children, ...restProps }: Props = $props()
</script>

<li {...restProps}>{@render children?.()}</li>
