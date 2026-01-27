<script lang="ts">
import { Select as SelectPrimitive } from 'bits-ui'

const { ...restProps }: SelectPrimitive.GroupProps = $props()
</script>

<SelectPrimitive.Group {...restProps} />
