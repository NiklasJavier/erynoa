<script lang="ts">
import PageContent from '$lib/components/PageContent.svelte'
import { onMount } from 'svelte'

let status = $state('Initial')
let apiResult = $state<string | null>(null)
let error = $state<string | null>(null)

onMount(async () => {
	status = 'Mounted'

	try {
		status = 'Testing fetch...'
		const response = await fetch(
			'http://localhost:3000/api/v1/connect/erynoa.v1.InfoService/GetInfo',
			{
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: '{}',
			}
		)

		if (!response.ok) {
			throw new Error(`HTTP ${response.status}`)
		}

		const data = await response.json()
		apiResult = JSON.stringify(data, null, 2)
		status = 'API call successful!'
	} catch (err) {
		error = err instanceof Error ? err.message : String(err)
		status = 'Failed'
	}
})
</script>

<PageContent>
	<div class="font-mono">
	<h1 class="text-2xl font-bold mb-4">API Test Page</h1>
	
	<div class="mb-4">
		<strong>Status:</strong> {status}
	</div>
	
	{#if error}
		<div class="p-4 bg-red-100 text-red-800 rounded mb-4">
			<strong>Error:</strong> {error}
		</div>
	{/if}
	
	{#if apiResult}
		<div class="p-4 bg-green-100 text-green-800 rounded">
			<strong>API Result:</strong>
			<pre class="mt-2 text-sm">{apiResult}</pre>
		</div>
		{/if}
	</div>
</PageContent>
