<script lang="ts">
import { authStore } from '$lib/auth'
import { Button } from '$lib/components/ui/button'
import * as Card from '$lib/components/ui/card'
import { Input } from '$lib/components/ui/input'
import { Label } from '$lib/components/ui/label'
import { Separator } from '$lib/components/ui/separator'
import { cn } from '$lib/utils'
import { Loader2 } from 'lucide-svelte'
import GalleryVerticalEnd from 'lucide-svelte/icons/gallery-vertical-end'

interface Props {
	class?: string
}

const { class: className }: Props = $props()
let isLoading = $state(false)

async function handleLogin() {
	isLoading = true
	try {
		await authStore.login()
	} finally {
		isLoading = false
	}
}
</script>

<!-- signup-03 / login-03 style: centered card with logo above -->
<div class={cn("bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10", className)}>
	<div class="flex w-full max-w-sm flex-col gap-6">
		<!-- Logo -->
		<a href="/" class="flex items-center gap-2 self-center font-medium">
			<div class="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md">
				<GalleryVerticalEnd class="size-4" />
			</div>
			Godstack
		</a>
		
		<!-- Login Card -->
		<Card.Root>
			<Card.Header class="text-center">
				<Card.Title class="text-xl">Welcome back</Card.Title>
				<Card.Description>Login with your Apple or Google account</Card.Description>
			</Card.Header>
			<Card.Content>
				<form onsubmit={(e) => { e.preventDefault(); handleLogin(); }}>
					<div class="flex flex-col gap-6">
						<!-- Social Login Buttons -->
						<div class="flex flex-col gap-4">
							<Button variant="outline" type="button" class="w-full" onclick={handleLogin} disabled={isLoading}>
								{#if isLoading}
									<Loader2 class="mr-2 h-4 w-4 animate-spin" />
									Signing in...
								{:else}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="mr-2 h-4 w-4">
										<path
											d="M12.152 6.896c-.948 0-2.415-1.078-3.96-1.04-2.04.027-3.91 1.183-4.961 3.014-2.117 3.675-.546 9.103 1.519 12.09 1.013 1.454 2.208 3.09 3.792 3.039 1.52-.065 2.09-.987 3.935-.987 1.831 0 2.35.987 3.96.948 1.637-.026 2.676-1.48 3.676-2.948 1.156-1.688 1.636-3.325 1.662-3.415-.039-.013-3.182-1.221-3.22-4.857-.026-3.04 2.48-4.494 2.597-4.559-1.429-2.09-3.623-2.324-4.39-2.376-2-.156-3.675 1.09-4.61 1.09zM15.53 3.83c.843-1.012 1.4-2.427 1.245-3.83-1.207.052-2.662.805-3.532 1.818-.78.896-1.454 2.338-1.273 3.714 1.338.104 2.715-.688 3.559-1.701"
											fill="currentColor"
										/>
									</svg>
									Login with Apple
								{/if}
							</Button>
							<Button variant="outline" type="button" class="w-full" onclick={handleLogin} disabled={isLoading}>
								<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="mr-2 h-4 w-4">
									<path
										d="M12.48 10.92v3.28h7.84c-.24 1.84-.853 3.187-1.787 4.133-1.147 1.147-2.933 2.4-6.053 2.4-4.827 0-8.6-3.893-8.6-8.72s3.773-8.72 8.6-8.72c2.6 0 4.507 1.027 5.907 2.347l2.307-2.307C18.747 1.44 16.133 0 12.48 0 5.867 0 .307 5.387.307 12s5.56 12 12.173 12c3.573 0 6.267-1.173 8.373-3.36 2.16-2.16 2.84-5.213 2.84-7.667 0-.76-.053-1.467-.173-2.053H12.48z"
										fill="currentColor"
									/>
								</svg>
								Login with Google
							</Button>
						</div>
						
						<!-- Separator -->
						<div class="relative text-center text-sm after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t after:border-border">
							<span class="relative z-10 bg-card px-2 text-muted-foreground">
								Or continue with
							</span>
						</div>
						
						<!-- Email/Password Form -->
						<div class="grid gap-4">
							<div class="grid gap-2">
								<Label for="email">Email</Label>
								<Input id="email" type="email" placeholder="m@example.com" required />
							</div>
							<div class="grid gap-2">
								<div class="flex items-center">
									<Label for="password">Password</Label>
									<a href="/forgot-password" class="ml-auto text-sm underline-offset-4 hover:underline">
										Forgot your password?
									</a>
								</div>
								<Input id="password" type="password" required />
							</div>
							<Button type="submit" class="w-full">
								Login
							</Button>
						</div>
						
						<!-- Sign up link -->
						<div class="text-center text-sm">
							Don't have an account?{' '}
							<a href="/signup" class="underline underline-offset-4">
								Sign up
							</a>
						</div>
					</div>
				</form>
			</Card.Content>
		</Card.Root>
	</div>
</div>
