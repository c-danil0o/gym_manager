<script lang="ts">
	import { onMount } from 'svelte';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { check, type Update } from '@tauri-apps/plugin-updater';
	import Button from '$lib/components/ui/button/button.svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { setHeader } from '$lib/stores/state';
	import { m } from '$lib/paraglide/messages';
	import { Download, RefreshCw, CheckCircle, AlertTriangle, Rocket } from 'lucide-svelte';

	let updateAvailable = $state(false);
	let error = $state<string | null>(null);
	let checking = $state(false);
	let installing = $state(false);
	let update: Update | null = $state(null);
	let downloadProgress = $state(0);
	let progressText = $state('');
	let downloadedBytes = $state(0);
	let totalBytes = $state(0);

	async function checkForUpdates() {
		checking = true;
		error = null;
		updateAvailable = false;

		try {
			const result = await check();
			if (result) {
				update = result;
				updateAvailable = true;
				console.log(`Update available: version ${result.version}`);
			} else {
				updateAvailable = false;
				update = null;
				console.log('You are on the latest version.');
			}
		} catch (e) {
			console.error(e);
			error = String(e);
		} finally {
			checking = false;
		}
	}

	async function startInstall() {
		if (!updateAvailable || !update) return;

		installing = true;
		downloadProgress = 0;
		downloadedBytes = 0;
		totalBytes = 0;

		try {
			await update.downloadAndInstall((event) => {
				switch (event.event) {
					case 'Started':
						totalBytes = event.data.contentLength || 0;
						progressText = 'Starting download...';
						console.log(`Started downloading ${event.data.contentLength} bytes`);
						break;
					case 'Progress':
						downloadedBytes += event.data.chunkLength;
						if (totalBytes > 0) {
							downloadProgress = Math.round((downloadedBytes / totalBytes) * 100);
						}
						progressText = `Downloading... ${Math.round(downloadedBytes / 1024 / 1024 * 100) / 100}MB / ${Math.round(totalBytes / 1024 / 1024 * 100) / 100}MB`;
						console.log(`Downloaded ${downloadedBytes} from ${totalBytes}`);
						break;
					case 'Finished':
						downloadProgress = 100;
						progressText = 'Download complete! Preparing to install...';
						console.log('Download finished');
						break;
				}
			});

			progressText = 'Installation complete! Relaunching...';
			await new Promise(resolve => setTimeout(resolve, 1000)); // Brief delay to show completion
			await relaunch();
		} catch (e) {
			error = `Installation failed: ${String(e)}`;
			installing = false;
		}
	}

	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 Bytes';
		const k = 1024;
		const sizes = ['Bytes', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	onMount(async () => {
		setHeader({
			title: m.updater(),
			showBackButton: true
		});
		await checkForUpdates();
	});
</script>

<div class="container max-w-2xl mx-auto p-4 space-y-6">
	<Card class="w-full">
		<CardHeader class="text-center">
			<div class="flex items-center justify-center gap-2 mb-2">
				<Rocket class="h-6 w-6 text-primary" />
				<CardTitle class="text-2xl">{m.app_updater()}</CardTitle>
			</div>
			<CardDescription>
				{m.updater_description()}
			</CardDescription>
		</CardHeader>

		<CardContent class="space-y-6">
			{#if checking}
				<div class="flex flex-col items-center gap-4 py-8">
					<div class="relative">
						<RefreshCw class="h-8 w-8 text-primary animate-spin" />
					</div>
					<p class="text-center text-muted-foreground">{m.checking_for_updates()}</p>
				</div>
			{:else if error}
				<Alert variant="destructive">
					<AlertTriangle class="h-4 w-4" />
					<AlertDescription>
						<strong>{m.error_checking_updates()}</strong> {error}
					</AlertDescription>
				</Alert>
				<div class="flex justify-center">
					<Button onclick={checkForUpdates} variant="outline">
						<RefreshCw class="h-4 w-4 mr-2" />
						{m.try_again()}
					</Button>
				</div>
			{:else if updateAvailable && update}
				<div class="space-y-4">
					<div class="flex items-center gap-2">
						<Badge variant="secondary" class="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300">
							{m.update_available()}
						</Badge>
						<span class="font-semibold">{m.version()} {update.version}</span>
					</div>

					{#if update.body}
						<div class="bg-muted/50 rounded-lg p-4">
							<h4 class="font-medium mb-2">{m.release_notes()}</h4>
							<div class="text-sm text-muted-foreground whitespace-pre-wrap">
								{update.body}
							</div>
						</div>
					{/if}

					{#if installing}
						<div class="space-y-4 py-4">
							<div class="flex items-center gap-2">
								<Download class="h-4 w-4 text-primary animate-bounce" />
								<span class="text-sm font-medium">{m.installing_update()}</span>
							</div>

							<div class="space-y-2">
								<Progress value={downloadProgress} class="w-full h-2" />
								<div class="flex justify-between text-xs text-muted-foreground">
									<span>{progressText}</span>
									<span>{downloadProgress}%</span>
								</div>
							</div>

							{#if totalBytes > 0}
								<div class="text-center text-xs text-muted-foreground">
									{formatFileSize(downloadedBytes)} / {formatFileSize(totalBytes)}
								</div>
							{/if}
						</div>
					{:else}
						<Button onclick={startInstall} class="w-full" size="lg">
							<Download class="h-4 w-4 mr-2" />
							{m.install_update_and_relaunch()}
						</Button>
					{/if}
				</div>
			{:else}
				<div class="flex flex-col items-center gap-4 py-8">
					<div class="flex items-center justify-center w-16 h-16 rounded-full bg-green-100 dark:bg-green-900">
						<CheckCircle class="h-8 w-8 text-green-600 dark:text-green-400" />
					</div>
					<div class="text-center space-y-2">
						<p class="font-medium">{m.up_to_date()}</p>
						<p class="text-sm text-muted-foreground">
							{m.running_latest_version()}
						</p>
					</div>
					<Button onclick={checkForUpdates} variant="outline">
						<RefreshCw class="h-4 w-4 mr-2" />
						{m.check_again()}
					</Button>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>
