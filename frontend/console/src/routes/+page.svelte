<script lang="ts">
	import PageContent from '$lib/components/PageContent.svelte';
	import SectionCards from '$lib/components/dashboard/SectionCards.svelte';
	import ChartAreaInteractive from '$lib/components/dashboard/ChartAreaInteractive.svelte';
	import DataTable from '$lib/components/dashboard/DataTable.svelte';
	import QuickLinks from '$lib/components/dashboard/QuickLinks.svelte';
</script>

<PageContent>
	<QuickLinks class="mb-6" />
	<SectionCards />
	<div class="mt-6">
		<ChartAreaInteractive />
	</div>
	<div class="mt-6">
		<DataTable />
	</div>
</PageContent>

