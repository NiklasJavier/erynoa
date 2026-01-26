<script lang="ts">
	import '../layout.css';
	import { onMount } from 'svelte';
	import { authStore } from '$lib/auth';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';

	let { children } = $props();
	
	onMount(() => {
		// Initialize auth for callback processing
		authStore.init();
	});
</script>

<ModeWatcher defaultMode="light" />
{@render children()}
<Toaster />
