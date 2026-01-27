<script lang="ts">
import '../layout.css'
import { authStore } from '$lib/auth'
import { Toaster } from '$lib/components/ui/sonner'
import { ModeWatcher } from 'mode-watcher'
import { onMount } from 'svelte'

const { children } = $props()

onMount(() => {
	// Initialize auth for callback processing
	authStore.init()
})
</script>

<ModeWatcher defaultMode="light" />
{@render children()}
<Toaster />
