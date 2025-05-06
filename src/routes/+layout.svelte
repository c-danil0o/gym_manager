<script lang="ts">
	import '../app.css';
	import { auth } from '$lib/stores/auth';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { navigating, page } from '$app/stores'; // To show loading during navigation
	import { goto } from '$app/navigation';
	import Login2 from '$lib/components/login/Login2.svelte';
	import { Toaster } from '$lib/components/ui/sonner';

	$: if (browser && !$auth.isAuthenticated && $page.url.pathname !== '/') {
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

{#if !mounted}
	<div>Loading App...</div>
{:else if $auth.isAuthenticated}
	<div class="app-shell">
		<header>
			<h1>Gym Management</h1>
			<nav>
				<!-- Add navigation links for authenticated users here -->
				<!-- e.g., <a href="/dashboard">Dashboard</a> | <a href="/members">Members</a> -->
				<button on:click={handleLogout}>Logout (User: {$auth.username})</button>
			</nav>
		</header>
		<main class="authenticated-content">
			<slot />
		</main>
		<footer>
			<p>© 2023 Gym Manager</p>
		</footer>
	</div>
{:else}
	<Login2 />
{/if}
<Toaster richColors theme="light" />

<style>
	.app-shell {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}
	header {
		background-color: #333;
		color: white;
		padding: 1em;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	header h1 {
		margin: 0;
		font-size: 1.5em;
	}
	nav button,
	nav a {
		color: white;
		margin-left: 1em;
		text-decoration: none;
		background: none;
		border: none;
		cursor: pointer;
		font-size: 1em;
	}
	nav button:hover,
	nav a:hover {
		text-decoration: underline;
	}
	.authenticated-content {
		flex-grow: 1;
		padding: 1em;
	}
	footer {
		text-align: center;
		padding: 1em;
		background-color: #f1f1f1;
		border-top: 1px solid #ddd;
	}

	/* Global styles can go in app.html or a separate CSS file imported here */
	:global(body) {
		margin: 0;
		font-family: Arial, sans-serif;
	}
</style>
