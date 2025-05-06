<script lang="ts">
	import { auth } from '$lib/stores/auth'; // Use $lib alias for SvelteKit
	// import { goto } from '$app/navigation'; // For redirecting after login

	let username = '';
	let password = '';
	let isLoading = false;
	let localError = '';

	async function handleSubmit() {
		if (!username || !password) {
			localError = 'Username and password are required.';
			return;
		}
		localError = '';
		isLoading = true;
		auth.clearError();

		const loginSuccess = await auth.login(username, password);
		isLoading = false;
	}

	$: if (username || password) localError = '';
</script>

<div class="login-container">
	<h2>Admin Login</h2>
	<form on:submit|preventDefault={handleSubmit}>
		<!-- ... (form fields and error messages same as before) ... -->
		<div class="form-group">
			<label for="username">Username:</label>
			<input type="text" id="username" bind:value={username} required disabled={isLoading} />
		</div>
		<div class="form-group">
			<label for="password">Password:</label>
			<input type="password" id="password" bind:value={password} required disabled={isLoading} />
		</div>

		{#if localError}
			<p class="error-message local-error">{localError}</p>
		{/if}
		{#if $auth.error && !isLoading}
			<p class="error-message auth-error">{$auth.error}</p>
		{/if}

		<button type="submit" disabled={isLoading}>
			{isLoading ? 'Logging in...' : 'Login'}
		</button>
	</form>
</div>

<style>
	/* ... (styles same as before) ... */
	.login-container {
		max-width: 400px;
		margin: 50px auto;
		padding: 20px;
		border: 1px solid #ccc;
		border-radius: 8px;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}
	.form-group {
		margin-bottom: 15px;
	}
	label {
		display: block;
		margin-bottom: 5px;
	}
	input[type='text'],
	input[type='password'] {
		width: 100%;
		padding: 8px;
		box-sizing: border-box;
		border: 1px solid #ddd;
		border-radius: 4px;
	}
	button {
		padding: 10px 15px;
		background-color: #007bff;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
	}
	button:disabled {
		background-color: #aaa;
	}
	.error-message {
		color: red;
		margin-top: 0;
		margin-bottom: 10px;
		font-size: 0.9em;
	}
</style>
