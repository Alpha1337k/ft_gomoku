<template>
	<div class="bg-black w-screen h-screen font-mono text-white">
		<div class="container max-w-7xl mx-auto p-8">
			<div class="flex justify-between">
				<div>
					<h1 class="text-2xl font-semibold">ft_gomoku</h1>
					<p class="text-sm">By okruitho</p>
				</div>
				<div v-if="gameState.wsOK === false" class="rounded bg-red-600 text-white flex items-center space-x-4 px-4">
					<ExclamationTriangleIcon class="h-10" />
					<p>Websocket disconnected.</p>
				</div>
				<div v-else-if="gameState.wsOK === undefined" class="rounded bg-yellow-600 text-white flex items-center space-x-4 px-4">
					<ArrowPathIcon class="h-10 p-2 animate-spin" />
					<p>Loading websocket.</p>
				</div>
			</div>

			<RouterView />
		</div>
	</div>
</template>

<script setup lang="ts">
import { useGameStateStore } from "./stores/GameState";
import { ArrowPathIcon, ExclamationTriangleIcon } from "@heroicons/vue/24/outline";

const gameState = useGameStateStore();
</script>
