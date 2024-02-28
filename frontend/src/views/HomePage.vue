<template>
	<div class="flex justify-center my-4 mt-52">
		<h2 class="text-xl">Select an game mode to begin.</h2>
	</div>

	<div class="flex space-x-2 justify-center">
		<RouterLink
			to="/hotseat"
			class="flex space-x-4 items-center w-64 bg-gradient-to-br cursor-pointer hover:to-green-600 transition from-green-500 to-green-700 rounded-lg p-4 text-lg font-semibold"
		>
			<UsersIcon class="h-6" />
			<p>Hotseat</p>
		</RouterLink>
		<RouterLink
			to="/bot"
			class="flex space-x-4 items-center w-64 bg-gradient-to-br cursor-pointer hover:to-amber-600 transition from-red-500 to-red-700 rounded-lg p-4 text-lg font-semibold"
		>
			<img src="/robot1.png" class="rounded-full h-12" />
			<p>Against AI</p>
		</RouterLink>
	</div>
</template>

<script setup lang="ts">
import { UsersIcon } from "@heroicons/vue/24/outline";
</script>
