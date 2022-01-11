
<script context="module" lang="ts">
</script>

<script lang="ts">
	import init, { Registration } from "authentication-wasm";
	
	let username = "";
	let email = "";
	let password = "";
	let loading = false;
	let successMessage = "";
	let errorMessage = "";

	async function register() {
		if (loading) return;
		if (!email || !password || !username) {
			errorMessage = "Please fill in all fields";
			return;
		}
		try{
			loading = true;
			errorMessage = "";
			successMessage = "";
			await init();
			const registration = new Registration(password);
			const serverStartResponse = await fetch("http://127.0.0.1:8787/register/start", {
				method: "POST",
				body: JSON.stringify({
					username: username,
					mail: email,
					request: registration.serverRequest
				}),
				headers: {
					"Content-Type": "application/json"
				}
			});
			if(!serverStartResponse.ok) {
				throw new Error("Server error");
			}
			const serverStart = await serverStartResponse.text();
			const registrationFinishServerRequest = registration.finish(username, serverStart);
			const serverFinishResponse = await fetch("http://127.0.0.1:8787/register/end", {
				method: "POST",
				body: JSON.stringify({
					username: username,
					mail: email,
					request: registrationFinishServerRequest
				}),
				headers: {
					"Content-Type": "application/json"
				}
			});
			if (serverFinishResponse.ok) {
				console.log("Registration successful!");
				successMessage = "Registration successful!";
			} else {
				console.error("Registration failed!");
				errorMessage = "Registration failed!";
			}
		} catch(e) {
			console.error("Registration failed!", e);
			errorMessage = `Registration failed with error: ${e.message}`;
		}
	}
</script>

<svelte:head>
	<title>Register</title>
</svelte:head>

<section>
	<div class="container mx-auto flex px-5 py-24 items-center justify-center flex-col">
		<h1 class="text-gray-900 text-xl mb-1 font-medium title-font">Register</h1>
		<div class="relative mb-4 w-1/3">
		  <label for="username" class="leading-7 text-sm text-gray-600">Username:</label>
		  <input disabled={loading} bind:value={username}  type="text" id="username" name="username" class="w-full bg-white rounded border border-gray-300 focus:border-yellow-500 focus:ring-2 focus:ring-yellow-200 text-base outline-none text-gray-700 py-1 px-3 leading-8 transition-colors duration-200 ease-in-out">
		</div>
		<div class="relative mb-4 w-1/3">
			<label for="email" class="leading-7 text-sm text-gray-600">Email:</label>
			<input disabled={loading} bind:value={email}  type="email" id="email" name="email" class="w-full bg-white rounded border border-gray-300 focus:border-yellow-500 focus:ring-2 focus:ring-yellow-200 text-base outline-none text-gray-700 py-1 px-3 leading-8 transition-colors duration-200 ease-in-out">
		  </div>
		<div class="relative mb-4 w-1/3">
		  <label for="password" class="leading-7 text-sm text-gray-600">Password:</label>
		  <input disabled={loading} bind:value={password}  type="password" id="password" name="password" class="w-full bg-white rounded border border-gray-300 focus:border-yellow-500 focus:ring-2 focus:ring-yellow-200 text-base outline-none text-gray-700 py-1 px-3 leading-8 transition-colors duration-200 ease-in-out">
		</div>

		<button disabled={loading} class="text-white bg-yellow-500 border-0 py-2 px-6 focus:outline-none hover:bg-yellow-600 rounded text-lg disabled:bg-gray-400" on:click={register}>Register</button>
		{#if errorMessage}
		<p class="text-red-500 mt-2">{errorMessage}</p>
		{/if}
		{#if successMessage}
		<p class="text-green-500 mt-2">{errorMessage}</p>
		{/if}
		<p class="text-xs text-gray-500 mt-3">Your password won't be sent over the network 🌍</p>
		<a class="text-yellow-500 focus:outline-none hover:text-yellow-600 mt-10" href="/">⏪ Go back</a>
	</div>
</section>
