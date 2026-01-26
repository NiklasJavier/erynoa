<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Home, ArrowLeft } from 'lucide-svelte';
</script>

<div class="flex items-center justify-center min-h-[60vh]">
	<Card.Root class="w-full max-w-md text-center">
		<Card.Header>
			<div class="text-6xl font-bold text-muted-foreground mb-4">404</div>
			<Card.Title>Page Not Found</Card.Title>
			<Card.Description>
				The page you're looking for doesn't exist or has been moved.
			</Card.Description>
		</Card.Header>
		<Card.Footer class="flex justify-center gap-4">
			<Button variant="outline" onclick={() => history.back()}>
				<ArrowLeft class="mr-2 h-4 w-4" />
				Go Back
			</Button>
			<Button href="/">
				<Home class="mr-2 h-4 w-4" />
				Home
			</Button>
		</Card.Footer>
	</Card.Root>
</div>
