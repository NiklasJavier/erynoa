<script lang="ts" module>
	import { Collapsible as CollapsiblePrimitive } from 'bits-ui';

	export type RootProps = CollapsiblePrimitive.RootProps;
	export type ContentProps = CollapsiblePrimitive.ContentProps;
	export type TriggerProps = CollapsiblePrimitive.TriggerProps;
</script>

<script lang="ts">
	import { cn } from '$lib/utils.js';

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: RootProps = $props();
</script>

<CollapsiblePrimitive.Root bind:ref class={cn(className)} {...restProps} />
