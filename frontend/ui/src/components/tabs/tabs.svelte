<script lang="ts">
import { cn } from '../../utils.js'
import { Tabs as TabsPrimitive } from 'bits-ui'

let { ref = $bindable(null), class: className, ...restProps }: TabsPrimitive.RootProps = $props()
</script>

<TabsPrimitive.Root bind:ref class={cn("", className)} {...restProps} />
