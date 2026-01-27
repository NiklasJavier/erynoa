<script lang="ts">
import { Dialog as SheetPrimitive } from 'bits-ui'

const { ...restProps }: SheetPrimitive.RootProps = $props()
</script>

<SheetPrimitive.Root {...restProps} />
