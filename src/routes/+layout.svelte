<script lang="ts">
	import CircleUser from 'lucide-svelte/icons/circle-user';
	import ChartLine from 'lucide-svelte/icons/chart-line';
	import Package from 'lucide-svelte/icons/package';
	import Scanner from 'lucide-svelte/icons/scan-barcode';
	import Menu from 'lucide-svelte/icons/menu';
	import Dumbbell from 'lucide-svelte/icons/dumbbell';
	import Log from 'lucide-svelte/icons/book-down';

	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import * as Sheet from '$lib/components/ui/sheet/index.js';
	import { headerState, loadingState, resetHeader } from '$lib/stores/state';
	import ArrowLeft from 'lucide-svelte/icons/arrow-left';

	import '../app.css';
	import { auth } from '$lib/stores/auth';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { page } from '$app/state'; // To show loading during navigation
	import { navigating } from '$app/stores';
	import { goto } from '$app/navigation';
	import Login from '$lib/components/login/login.svelte';
	import { Toaster } from '$lib/components/ui/sonner';
	import { ModeWatcher, setMode } from 'mode-watcher';
	import { Firework } from 'svelte-loading-spinners';
	import LightSwitch from '$lib/components/light-switch/light-switch.svelte';
	import { User } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';
	import { isLocale, setLocale, getLocale } from '$lib/paraglide/runtime.js';
	import { listen } from '@tauri-apps/api/event';
	import { m } from '$lib/paraglide/messages';

	let { children } = $props();
	$effect(() => {
		if (browser && !$auth.isAuthenticated && page.url.pathname !== '/') {
			goto('/');
			resetHeader();
		}
	});
	function handleBack() {
		if ($headerState.backPath) {
			goto($headerState.backPath);
		} else {
			window.history.back();
		}
	}

	let mounted = $state(false);

	function handleLogout() {
		auth.logout();
		goto('/');
	}

	interface AppSettings {
		language: string;
		theme: string;
		timezone: string;
		backup_url?: string | null;
		backup_period_hours: number;
	}

	async function loadAndApplySettings() {
		try {
			const settings = await invoke<AppSettings>('get_app_settings');
			console.log('App settings loaded:', settings);

			if (settings.language && isLocale(settings.language)) {
				if (getLocale() !== settings.language) {
					setLocale(settings.language, { reload: false });
				}
			}
			if (settings.theme === 'light' || settings.theme === 'dark') {
				setMode(settings.theme as 'light' | 'dark');
			}
		} catch (e: any) {
			console.log(e);
			toast.error(m['main.toast_failed_settings']());
		}
	}

	onMount(() => {
		let unlisten: () => void;
		async function init() {
			await loadAndApplySettings();

			unlisten = await listen<AppSettings>('settings_changed', (event) => {
				console.log('Settings changed event received:', event.payload);
				toast.info(m['main.toast_settings_updated']());
				if (event.payload.language && isLocale(event.payload.language)) {
					setLocale(event.payload.language);
				}
				if (event.payload.theme) {
					setMode(event.payload.theme as 'light' | 'dark' | 'system');
				}
			});
			mounted = true;
		}
		init();

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});
</script>

{#if $navigating || $loadingState}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/50">
		<Firework size="100" color="gray" unit="px" duration="1s" />
	</div>
{/if}
{#if !mounted}
	<div>Loading App...</div>
{:else if $auth.isAuthenticated}
	<div
		class="grid min-h-screen w-full bg-sidebar md:grid-cols-[220px_1fr] lg:grid-cols-[280px_1fr]"
	>
		<div class="hidden md:block">
			<div class="flex h-full max-h-screen flex-col gap-2">
				<div class="flex h-14 items-center mx-1.5 px-4 my-1.5 border-b lg:h-[60px] lg:px-6">
					<a href="/" class="flex items-center gap-2 font-semibold">
						<Dumbbell class="h-6 w-6" />
						<span class="">Aka Gym</span>
					</a>
				</div>
				<div class="flex-1 overflow-y-auto">
					<nav class="grid items-start bg-sidebar px-2 space-y-4 text-sm font-medium lg:px-4">
						<a
							href="/"
							class="{page.url.pathname === '/'
								? 'bg-background/60 border shadow-sm text-foreground'
								: ''} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-xl px-3 py-2 transition-all"
						>
							<Scanner class="h-4 w-4" />
							{m['common.scanner']()}
						</a>
						<a
							href="/members"
							class="{page.url.pathname === '/members' || page.url.pathname.startsWith('/members/')
								? 'bg-background/60 border shadow-sm text-foreground'
								: ''} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-xl px-3 py-2 transition-all"
						>
							<User class="h-4 w-4" />
							{m['common.members']()}
						</a>
						<a
							href="/memberships"
							class="{page.url.pathname.startsWith('/memberships')
								? 'bg-background/60 border shadow-sm text-foreground'
								: ''} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-xl px-3 py-2 transition-all"
						>
							<Package class="h-4 w-4" />
							{m['common.memberships']()}
						</a>
						<a
							href="/entry-log"
							class="{page.url.pathname.startsWith('/entry-log')
								? 'bg-background/60 border shadow-sm text-foreground'
								: ''} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-xl px-3 py-2 transition-all"
						>
							<Log class="h-4 w-4" />
							{m['common.entry_log']()}
						</a>
						<a
							href="/analytics"
							class="{page.url.pathname.startsWith('/analytics')
								? 'bg-background/60 border shadow-sm text-foreground'
								: ''} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-xl px-3 py-2 transition-all"
						>
							<ChartLine class="h-4 w-4" />
							{m['common.analytics']()}
						</a>
					</nav>
				</div>
				<div class="mt-auto p-4 shrink-0">
					<Card.Root>
						<Card.Header class="p-2 pt-0 md:p-4">
							<Card.Title>{m['main.add_new_member']()}</Card.Title>
							<Card.Description>{m['main.add_new_member_desc']()}</Card.Description>
						</Card.Header>
						<Card.Content class="p-2 pt-0 md:p-4 md:pt-0">
							<Button
								size="sm"
								class="w-full"
								onclick={() => {
									goto('/members/new');
								}}>{m['common.add']()}</Button
							>
						</Card.Content>
					</Card.Root>
				</div>
			</div>
		</div>
		<div class="flex flex-col h-screen mr-1.5">
			<header
				class="flex bg-background rounded-t-2xl mt-1.5 h-14 items-center gap-4 border shadow px-4 lg:h-[60px] lg:px-6"
			>
				<Sheet.Root>
					<Sheet.Trigger>
						<Button variant="outline" size="icon" class="shrink-0 md:hidden">
							<Menu class="h-5 w-5" />
							<span class="sr-only">{m['main.menu_toggle']()}</span>
						</Button>
					</Sheet.Trigger>
					<Sheet.Content side="left" class="flex flex-col">
						<nav class="grid gap-2 text-lg font-medium">
							<a href="##" class="flex items-center gap-2 mb-3 text-lg font-semibold">
								<Dumbbell class="h-6 w-6" />
								<span>Aka Gym</span>
							</a>
							<a
								href="/"
								class="{page.url.pathname === '/'
									? 'bg-background/60 border shadow-sm text-foreground'
									: ''} text-muted-foreground hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<Scanner class="h-5 w-5" />
								{m['common.scanner']()}
							</a>
							<a
								href="/members"
								class="{page.url.pathname.startsWith('/members/') ||
								page.url.pathname === '/members'
									? 'bg-background/60 border shadow-sm text-foreground'
									: ''} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<User class="h-5 w-5" />
								{m['common.members']()}
							</a>
							<a
								href="/memberships"
								class="{page.url.pathname.startsWith('/memberships')
									? 'bg-background/60 border shadow-sm text-foreground'
									: ''} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<Package class="h-5 w-5" />
								{m['common.memberships']()}
							</a>
							<a
								href="/entry-log"
								class="{page.url.pathname.startsWith('/members')
									? 'bg-background/60 border shadow-sm text-foreground'
									: ''} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<Log class="h-5 w-5" />
								{m['common.entry_log']()}
							</a>
							<a
								href="/analytics"
								class="{page.url.pathname.startsWith('/analytics')
									? 'bg-background/60 border shadow-sm text-foreground'
									: ''} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<ChartLine class="h-5 w-5" />
								{m['common.analytics']()}
							</a>
						</nav>
						<div class="mt-auto">
							<Card.Root>
								<Card.Header>
									<Card.Title>{m['main.add_new_member']()}</Card.Title>
									<Card.Description>{m['main.add_new_member_desc']()}</Card.Description>
								</Card.Header>
								<Card.Content>
									<Button
										size="sm"
										class="w-full"
										onclick={() => {
											goto('/members/new');
										}}>Add</Button
									>
								</Card.Content>
							</Card.Root>
						</div>
					</Sheet.Content>
				</Sheet.Root>
				<div class="flex flex-1 items-center gap-6">
					{#if $headerState.showBackButton}
						<Button variant="secondary" size="icon" onclick={handleBack} aria-label="Go back">
							<ArrowLeft class="h-5 w-5" />
						</Button>
					{:else}
						<div class="w-[32px] h-5"></div>
					{/if}
					<h1 class="text-lg font-semibold md:text-xl">{$headerState.title}</h1>
				</div>

				<LightSwitch />
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						<Button variant="secondary" size="icon" class="rounded-full">
							<CircleUser class="h-5 w-5" />
							<span class="sr-only">{m['main.user_toggle']()}</span>
						</Button>
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end">
						<DropdownMenu.Item onclick={() => goto('/settings')}>{m['common.settings']()}</DropdownMenu.Item>
						<DropdownMenu.Item>Update</DropdownMenu.Item>
						<DropdownMenu.Separator />
						<DropdownMenu.Item onclick={handleLogout}>{m['common.logout']()}</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</header>
			<main
				class="flex flex-1 flex-col gap-4 mb-1.5 rounded-b-2xl border-x shadow p-6 lg:gap-6 lg:p-10 overflow-y-auto bg-background"
			>
				{@render children?.()}
			</main>
		</div>
	</div>
{:else}
	<Login />
{/if}
<Toaster richColors theme="light" />
<ModeWatcher />
