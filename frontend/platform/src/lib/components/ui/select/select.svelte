<script lang="ts">
import { Select as SelectPrimitive } from 'bits-ui'

const { ...restProps }: SelectPrimitive.RootProps = $props()
</script>

<SelectPrimitive.Root {...restProps} />
