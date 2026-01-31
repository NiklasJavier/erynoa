<script lang="ts">
import { authStore } from '$lib/auth'
import { Toaster } from '@erynoa/ui/components/sonner'
import { ModeWatcher } from 'mode-watcher'
import { onMount } from 'svelte'
import '../layout.css'

const { children } = $props()

onMount(() => {
	// Initialize auth for callback processing
	authStore.init()
})
</script>

<ModeWatcher defaultMode="light" />
{@render children()}
<Toaster />
