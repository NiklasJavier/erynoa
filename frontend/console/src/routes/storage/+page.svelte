<script lang="ts">
import { goto } from '$app/navigation'
import { authStore, isAuthenticated, isLoading } from '$lib/auth'
import PageContent from '$lib/components/PageContent.svelte'
import { Badge } from '$lib/components/ui/badge'
import { Button } from '$lib/components/ui/button'
import * as Card from '$lib/components/ui/card'
import * as Dialog from '$lib/components/ui/dialog'
import { Input } from '$lib/components/ui/input'
import { Separator } from '$lib/components/ui/separator'
import {
	Download,
	File as FileIcon,
	Folder,
	FolderPlus,
	Loader2,
	RefreshCw,
	Trash2,
	Upload,
} from 'lucide-svelte'
import { onMount } from 'svelte'
import { toast } from 'svelte-sonner'

// Types
interface StorageObject {
	key: string
	size: number
	lastModified?: Date
	etag?: string
}

// State
let objects = $state<StorageObject[]>([])
let buckets = $state<string[]>([])
let currentBucket = $state<string>('uploads')
let isLoadingObjects = $state(false)
let isLoadingBuckets = $state(false)
let uploadDialogOpen = $state(false)
let createBucketDialogOpen = $state(false)
let newBucketName = $state('')
let selectedFile = $state<globalThis.File | null>(null)

// Redirect if not authenticated
$effect(() => {
	if (!$isLoading && !$isAuthenticated) {
		goto('/')
	}
})

onMount(() => {
	if ($isAuthenticated) {
		loadBuckets()
		loadObjects()
	}
})

async function loadBuckets() {
	isLoadingBuckets = true
	try {
		const { createAuthenticatedClients } = await import('$lib/api/clients')
		const clients = createAuthenticatedClients(() => authStore.getAccessToken())
		const response = await clients.storage.listBuckets({})
		buckets = response.buckets || []
	} catch (err) {
		console.error('Failed to load buckets:', err)
		toast.error('Failed to load buckets')
	} finally {
		isLoadingBuckets = false
	}
}

async function loadObjects() {
	isLoadingObjects = true
	try {
		const { createAuthenticatedClients } = await import('$lib/api/clients')
		const clients = createAuthenticatedClients(() => authStore.getAccessToken())
		const response = await clients.storage.list({ bucket: currentBucket })
		objects = (response.objects || []).map(
			(obj: { key: string; size: bigint; lastModified?: { toDate(): Date }; etag?: string }) => ({
				key: obj.key,
				size: Number(obj.size),
				lastModified: obj.lastModified?.toDate(),
				etag: obj.etag,
			})
		)
	} catch (err) {
		console.error('Failed to load objects:', err)
		toast.error('Failed to load objects')
	} finally {
		isLoadingObjects = false
	}
}

async function uploadFile() {
	if (!selectedFile) return

	try {
		const { createAuthenticatedClients } = await import('$lib/api/clients')
		const clients = createAuthenticatedClients(() => authStore.getAccessToken())

		// Get presigned upload URL
		const response = await clients.storage.getPresignedUploadUrl({
			bucket: currentBucket,
			key: selectedFile.name,
			contentType: selectedFile.type || 'application/octet-stream',
		})

		// Upload to presigned URL
		await fetch(response.url, {
			method: 'PUT',
			body: selectedFile,
			headers: {
				'Content-Type': selectedFile.type || 'application/octet-stream',
			},
		})

		toast.success('File uploaded successfully')
		uploadDialogOpen = false
		selectedFile = null
		loadObjects()
	} catch (err) {
		console.error('Upload failed:', err)
		toast.error('Upload failed')
	}
}

async function downloadFile(key: string) {
	try {
		const { createAuthenticatedClients } = await import('$lib/api/clients')
		const clients = createAuthenticatedClients(() => authStore.getAccessToken())

		const response = await clients.storage.getPresignedDownloadUrl({
			bucket: currentBucket,
			key,
		})

		window.open(response.url, '_blank')
	} catch (err) {
		console.error('Download failed:', err)
		toast.error('Download failed')
	}
}

async function deleteFile(key: string) {
	if (!confirm(`Delete ${key}?`)) return

	try {
		const { createAuthenticatedClients } = await import('$lib/api/clients')
		const clients = createAuthenticatedClients(() => authStore.getAccessToken())

		await clients.storage.delete({
			bucket: currentBucket,
			key,
		})

		toast.success('File deleted')
		loadObjects()
	} catch (err) {
		console.error('Delete failed:', err)
		toast.error('Delete failed')
	}
}

async function createBucket() {
	if (!newBucketName.trim()) return

	try {
		const { createAuthenticatedClients } = await import('$lib/api/clients')
		const clients = createAuthenticatedClients(() => authStore.getAccessToken())

		await clients.storage.createBucket({ name: newBucketName.trim() })

		toast.success('Bucket created')
		createBucketDialogOpen = false
		newBucketName = ''
		loadBuckets()
	} catch (err) {
		console.error('Create bucket failed:', err)
		toast.error('Failed to create bucket')
	}
}

function formatFileSize(bytes: number): string {
	if (bytes === 0) return '0 B'
	const k = 1024
	const sizes = ['B', 'KB', 'MB', 'GB']
	const i = Math.floor(Math.log(bytes) / Math.log(k))
	return `${Number.parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`
}

function handleFileSelect(event: Event) {
	const input = event.target as HTMLInputElement
	selectedFile = input.files?.[0] || null
}
</script>

{#if $isLoading}
	<div class="flex items-center justify-center min-h-[60vh]">
		<Loader2 class="h-8 w-8 animate-spin" />
	</div>
{:else if $isAuthenticated}
<PageContent>
	{#snippet headerActions()}
		<Button variant="outline" onclick={() => createBucketDialogOpen = true}>
			<FolderPlus class="mr-2 h-4 w-4" />
			New Bucket
		</Button>
		<Button onclick={() => uploadDialogOpen = true}>
			<Upload class="mr-2 h-4 w-4" />
			Upload
		</Button>
	{/snippet}

	<!-- Bucket Selector -->
	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center justify-between">
				<span>Buckets</span>
				<Button variant="ghost" size="icon" onclick={loadBuckets}>
					<RefreshCw class={`h-4 w-4 ${isLoadingBuckets ? 'animate-spin' : ''}`} />
				</Button>
			</Card.Title>
		</Card.Header>
		<Card.Content>
			<div class="flex flex-wrap gap-2">
				{#each buckets as bucket}
					<Badge
						variant={bucket === currentBucket ? 'default' : 'outline'}
						class="cursor-pointer"
						onclick={() => { currentBucket = bucket; loadObjects(); }}
					>
						<Folder class="mr-1 h-3 w-3" />
						{bucket}
					</Badge>
				{/each}
				{#if buckets.length === 0 && !isLoadingBuckets}
					<p class="text-muted-foreground text-sm">No buckets found</p>
				{/if}
			</div>
		</Card.Content>
	</Card.Root>

	<!-- Objects List -->
	<Card.Root class="mt-6">
		<Card.Header>
			<Card.Title class="flex items-center justify-between">
				<span>Files in {currentBucket}</span>
				<Button variant="ghost" size="icon" onclick={loadObjects}>
					<RefreshCw class={`h-4 w-4 ${isLoadingObjects ? 'animate-spin' : ''}`} />
				</Button>
			</Card.Title>
		</Card.Header>
		<Card.Content>
			{#if isLoadingObjects}
				<div class="flex items-center justify-center py-8">
					<Loader2 class="h-6 w-6 animate-spin" />
				</div>
			{:else if objects.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					<FileIcon class="h-12 w-12 mx-auto mb-4 opacity-50" />
					<p>No files in this bucket</p>
					<Button variant="link" onclick={() => uploadDialogOpen = true}>
						Upload your first file
					</Button>
				</div>
			{:else}
				<div class="space-y-2">
					{#each objects as obj}
						<div class="flex items-center justify-between p-3 rounded-lg border hover:bg-muted/50">
							<div class="flex items-center gap-3">
								<FileIcon class="h-5 w-5 text-muted-foreground" />
								<div>
									<p class="font-medium">{obj.key}</p>
									<p class="text-sm text-muted-foreground">
										{formatFileSize(obj.size)}
											{#if obj.lastModified}
												• {obj.lastModified.toLocaleDateString()}
											{/if}
										</p>
									</div>
								</div>
								<div class="flex gap-1">
									<Button variant="ghost" size="icon" onclick={() => downloadFile(obj.key)}>
										<Download class="h-4 w-4" />
									</Button>
									<Button variant="ghost" size="icon" onclick={() => deleteFile(obj.key)}>
										<Trash2 class="h-4 w-4 text-destructive" />
									</Button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</Card.Content>
		</Card.Root>

	<!-- Upload Dialog -->
	<Dialog.Root bind:open={uploadDialogOpen}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Upload File</Dialog.Title>
				<Dialog.Description>
					Select a file to upload to {currentBucket}
				</Dialog.Description>
			</Dialog.Header>
			<div class="py-4">
				<Input
					type="file"
					onchange={handleFileSelect}
				/>
				{#if selectedFile}
					<p class="text-sm text-muted-foreground mt-2">
						Selected: {selectedFile.name} ({formatFileSize(selectedFile.size)})
					</p>
				{/if}
			</div>
			<Dialog.Footer>
				<Button variant="outline" onclick={() => uploadDialogOpen = false}>
					Cancel
				</Button>
				<Button onclick={uploadFile} disabled={!selectedFile}>
					Upload
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<!-- Create Bucket Dialog -->
	<Dialog.Root bind:open={createBucketDialogOpen}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Create Bucket</Dialog.Title>
				<Dialog.Description>
					Enter a name for the new bucket
				</Dialog.Description>
			</Dialog.Header>
			<div class="py-4">
				<Input
					placeholder="bucket-name"
					bind:value={newBucketName}
				/>
			</div>
			<Dialog.Footer>
				<Button variant="outline" onclick={() => createBucketDialogOpen = false}>
					Cancel
				</Button>
				<Button onclick={createBucket} disabled={!newBucketName.trim()}>
					Create
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
</PageContent>
{/if}