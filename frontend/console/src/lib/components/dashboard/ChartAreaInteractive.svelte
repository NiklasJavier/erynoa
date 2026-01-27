<script lang="ts">
import { Button } from '$lib/components/ui/button'
import * as Card from '$lib/components/ui/card'
import * as Select from '$lib/components/ui/select'

let timeRange = $state('90d')

// Simulated chart data
const chartData = [
	{ date: '2024-01-01', desktop: 186, mobile: 80 },
	{ date: '2024-01-15', desktop: 305, mobile: 200 },
	{ date: '2024-02-01', desktop: 237, mobile: 120 },
	{ date: '2024-02-15', desktop: 203, mobile: 190 },
	{ date: '2024-03-01', desktop: 209, mobile: 130 },
	{ date: '2024-03-15', desktop: 264, mobile: 140 },
	{ date: '2024-04-01', desktop: 224, mobile: 180 },
]

const _filteredData = $derived(() => {
	const now = new Date('2024-04-01')
	const days = timeRange === '90d' ? 90 : timeRange === '30d' ? 30 : 7
	const cutoff = new Date(now.getTime() - days * 24 * 60 * 60 * 1000)
	return chartData.filter((d) => new Date(d.date) >= cutoff)
})
</script>

<Card.Root class="@container/card">
	<Card.Header class="flex flex-col space-y-0 border-b py-5 sm:flex-row sm:items-center sm:justify-between sm:py-6">
		<div class="flex flex-1 flex-col justify-center gap-1">
			<Card.Title>Area Chart - Interactive</Card.Title>
			<Card.Description>
				Showing total visitors for the last 3 months
			</Card.Description>
		</div>
		<div>
			<Select.Root type="single" value={timeRange} onValueChange={(v) => v && (timeRange = v)}>
				<Select.Trigger class="w-[160px] rounded-lg sm:ml-auto" aria-label="Select a value">
					<span>
						{timeRange === '90d' ? 'Last 3 months' : timeRange === '30d' ? 'Last 30 days' : 'Last 7 days'}
					</span>
				</Select.Trigger>
				<Select.Content class="rounded-xl">
					<Select.Item value="90d" class="rounded-lg">Last 3 months</Select.Item>
					<Select.Item value="30d" class="rounded-lg">Last 30 days</Select.Item>
					<Select.Item value="7d" class="rounded-lg">Last 7 days</Select.Item>
				</Select.Content>
			</Select.Root>
		</div>
	</Card.Header>
	<Card.Content class="px-2 pt-4 sm:px-6 sm:pt-6">
		<!-- Placeholder for chart - would use layerchart in production -->
		<div class="aspect-auto h-[250px] w-full flex items-center justify-center rounded-lg bg-muted/50">
			<div class="text-center text-muted-foreground">
				<svg
					class="mx-auto h-12 w-12 mb-4"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z"
					/>
				</svg>
				<p class="text-sm font-medium">Interactive Area Chart</p>
				<p class="text-xs mt-1">Desktop vs Mobile visitors</p>
				<p class="text-xs mt-2">Install layerchart for chart rendering</p>
			</div>
		</div>
	</Card.Content>
</Card.Root>
