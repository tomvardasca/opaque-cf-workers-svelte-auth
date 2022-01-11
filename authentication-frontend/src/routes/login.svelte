<script lang="ts">
	import init, { Login } from "authentication-wasm";
	
	let username = "";
	let password = "";
	let errorMessage = "";
	let successMessage = "";
	let loading = false;

	async function login() {
		if (loading) return;
		if (!password || !username) {
			errorMessage = "Please fill in all fields";
			return;
		}
		try {
			loading = true;
			errorMessage = "";
			successMessage = "";
			await init();
			const login = new Login(username, password);
			const serverStartResponse = await fetch("http://127.0.0.1:8787/login/start", {
				method: "POST",
				body: JSON.stringify({
					username: username,
					request: login.serverRequest
				}),
				headers: {
					"Content-Type": "application/json"
				}
			});
			if(!serverStartResponse.ok) {
				throw new Error("Server error");
			}
			const serverStart = await serverStartResponse.text();
			const {serverRequest: finalServerRequest, sessionKey} = login.finish(username, serverStart);
			const serverFinishResponse = await fetch("http://127.0.0.1:8787/login/end", {
				method: "POST",
				body: JSON.stringify({
					username: username,
					request: finalServerRequest
				}),
				headers: {
					"Content-Type": "application/json"
				}
			});
			if(!serverFinishResponse.ok) {
				throw new Error("Server error");
			}
			const serverFinish = await serverFinishResponse.text();
			if (serverFinish === sessionKey) {
				console.log("Login successful!", sessionKey);
				successMessage = "Login successful!";
			} else {
				console.error("Login failed!");
				errorMessage = "Login failed!";
			}
		} catch(e) {
			console.log("Login failed!", e);
			errorMessage = `Login failed with: ${e.message}`;
		}
		loading = false;
	}
</script>

<svelte:head>
	<title>Login</title>
</svelte:head>

<section>
	<div class="container mx-auto flex px-5 py-24 items-center justify-center flex-col">
		<h1 class="text-gray-900 text-xl mb-1 font-medium title-font">Login</h1>
		<div class="relative mb-4 w-1/3">
		  <label for="username" class="leading-7 text-sm text-gray-600">Username:</label>
		  <input disabled={loading} bind:value={username}  type="text" id="username" name="username" class="w-full bg-white rounded border border-gray-300 focus:border-yellow-500 focus:ring-2 focus:ring-yellow-200 text-base outline-none text-gray-700 py-1 px-3 leading-8 transition-colors duration-200 ease-in-out">
		</div>
		<div class="relative mb-4 w-1/3">
		  <label for="password" class="leading-7 text-sm text-gray-600">Password:</label>
		  <input disabled={loading} bind:value={password}  type="password" id="password" name="password" class="w-full bg-white rounded border border-gray-300 focus:border-yellow-500 focus:ring-2 focus:ring-yellow-200 text-base outline-none text-gray-700 py-1 px-3 leading-8 transition-colors duration-200 ease-in-out">
		</div>

		<button disabled={loading} class="text-white bg-yellow-500 border-0 py-2 px-6 focus:outline-none hover:bg-yellow-600 rounded text-lg disabled:bg-gray-400" on:click={login}>Login</button>
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


