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

	import '../app.css';
	import { auth } from '$lib/stores/auth';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { navigating, page } from '$app/state'; // To show loading during navigation
	import { goto } from '$app/navigation';
	import Login from '$lib/components/login/login.svelte';
	import { Toaster } from '$lib/components/ui/sonner';
	import { ModeWatcher } from 'mode-watcher';
	import { SpinLine } from 'svelte-loading-spinners';
	import LightSwitch from '$lib/components/light-switch/light-switch.svelte';
	import { User } from 'lucide-svelte';

	$: if (browser && !$auth.isAuthenticated && page.url.pathname !== '/') {
		goto('/');
	}

	let mounted = false;
	onMount(() => {
		mounted = true;
	});

	function handleLogout() {
		auth.logout();
		goto('/');
	}
</script>

{#if navigating.complete}
	<SpinLine size="60" color="#99c1f1" unit="px" duration="2s" />
{/if}

{#if !mounted}
	<div>Loading App...</div>
{:else if $auth.isAuthenticated}
	<div class="grid min-h-screen w-full md:grid-cols-[220px_1fr] lg:grid-cols-[280px_1fr]">
		<div class="bg-muted/20 hidden border-r md:block">
			<div class="flex h-full max-h-screen flex-col gap-2">
				<div class="flex h-14 items-center border-b px-4 lg:h-[60px] lg:px-6">
					<a href="/" class="flex items-center gap-2 font-semibold">
						<Dumbbell class="h-6 w-6" />
						<span class="">Aka Gym</span>
					</a>
				</div>
				<div class="flex-1">
					<nav class="grid items-start px-2 space-y-3 text-sm font-medium lg:px-4">
						<a
							href="/"
							class="{page.url.pathname === '/' ? 'bg-muted text-primary' : 'text-muted-foreground'} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-lg px-3 py-2 transition-all"
						>
							<Scanner class="h-4 w-4" />
							Scanner
						</a>
						<a
							href="/member-ships"
							class="{page.url.pathname.startsWith('/member-ships') ? 'bg-muted text-foreground' : 'text-muted-foreground'} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-lg px-3 py-2 transition-all"
						>
							<Package class="h-4 w-4" />
							Memberships
						</a>
						<a
							href="/members"
							class="{page.url.pathname.startsWith('/members') ? 'bg-muted text-foreground' : 'text-muted-foreground'} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-lg px-3 py-2 transition-all"
						>
							<User class="h-4 w-4" />
							Members
						</a>
						<a
							href="/entry-log"
							class="{page.url.pathname.startsWith('/entry-log') ? 'bg-muted text-foreground' : 'text-muted-foreground'} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-lg px-3 py-2 transition-all"
						>
							<Log class="h-4 w-4" />
							Entry Log
						</a>
						<a
							href="/analytics"
							class="{page.url.pathname.startsWith('/analytics') ? 'bg-muted text-primary' : 'text-muted-foreground'} hover:bg-accent-foreground hover:text-accent flex items-center gap-3 rounded-lg px-3 py-2 transition-all"
						>
							<ChartLine class="h-4 w-4" />
							Analytics
						</a>
					</nav>
				</div>
				<div class="mt-auto p-4">
					<Card.Root>
						<Card.Header class="p-2 pt-0 md:p-4">
							<Card.Title>Add new member</Card.Title>
							<Card.Description>Create new member and assign him a membership.</Card.Description>
						</Card.Header>
						<Card.Content class="p-2 pt-0 md:p-4 md:pt-0">
							<Button size="sm" class="w-full">Add</Button>
						</Card.Content>
					</Card.Root>
				</div>
			</div>
		</div>
		<div class="flex flex-col">
			<header class="bg-muted/40 flex h-14 items-center gap-4 border-b px-4 lg:h-[60px] lg:px-6">
				<Sheet.Root>
					<Sheet.Trigger asChild let:builder>
						<Button variant="outline" size="icon" class="shrink-0 md:hidden" builders={[builder]}>
							<Menu class="h-5 w-5" />
							<span class="sr-only">Toggle navigation menu</span>
						</Button>
					</Sheet.Trigger>
					<Sheet.Content side="left" class="flex flex-col">
						<nav class="grid gap-2 text-lg font-medium">
							<a href="##" class="flex items-center gap-2 text-lg font-semibold">
								<Dumbbell class="h-6 w-6" />
								<span class="sr-only">Aka Gym</span>
							</a>
							<a
								href="/"
								class="{page.url.pathname === '/' ? 'bg-muted text-foreground' : 'text-muted-foreground'} text-muted-foreground hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<Scanner class="h-5 w-5" />
								Scanner
							</a>
							<a
								href="/member-ships"
								class="{page.url.pathname.startsWith('/member-ships') ? 'bg-muted text-foreground' : 'text-muted-foreground'} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<Package class="h-5 w-5" />
								Memberships
							</a>
							<a
								href="/members"
								class="{page.url.pathname.startsWith('/members') ? 'bg-muted text-foreground' : 'text-muted-foreground'} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<User class="h-5 w-5" />
								Members
							</a>
							<a
								href="/entry-log"
								class="{page.url.pathname.startsWith('/members') ? 'bg-muted text-foreground' : 'text-muted-foreground'} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<Log class="h-5 w-5" />
								Entry Log
							</a>
							<a
								href="/analytics"
								class="{page.url.pathname.startsWith('/analytics') ? 'bg-muted text-foreground' : 'text-muted-foreground'} hover:text-foreground mx-[-0.65rem] flex items-center gap-4 rounded-xl px-3 py-2"
							>
								<ChartLine class="h-5 w-5" />
								Analytics
							</a>
						</nav>
						<div class="mt-auto">
							<Card.Root>
								<Card.Header>
									<Card.Title>Add Member</Card.Title>
									<Card.Description>Create new member and assign him a membership</Card.Description>
								</Card.Header>
								<Card.Content>
									<Button size="sm" class="w-full">Add</Button>
								</Card.Content>
							</Card.Root>
						</div>
					</Sheet.Content>
				</Sheet.Root>
				<div class="w-full flex-1"></div>
				<LightSwitch />
				<DropdownMenu.Root>
					<DropdownMenu.Trigger asChild let:builder>
						<Button builders={[builder]} variant="secondary" size="icon" class="rounded-full">
							<CircleUser class="h-5 w-5" />
							<span class="sr-only">Toggle user menu</span>
						</Button>
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end">
						<DropdownMenu.Label>My Account</DropdownMenu.Label>
						<DropdownMenu.Separator />
						<DropdownMenu.Item>Settings</DropdownMenu.Item>
						<DropdownMenu.Item>Update</DropdownMenu.Item>
						<DropdownMenu.Separator />
						<DropdownMenu.Item on:click={handleLogout}>Logout</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</header>
			<main class="flex flex-1 flex-col gap-4 p-4 lg:gap-6 lg:p-6">
				<slot />
			</main>
		</div>
	</div>
{:else}
	<Login />
{/if}
<Toaster richColors theme="light" />
<ModeWatcher />
