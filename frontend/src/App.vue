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
import GoBoard from "@/components/GoBoard.vue";
import SidePanel from "@/components/SidePanel.vue";
import { useGameStateStore } from "./stores/GameState";
import { computed, ref, watchEffect } from "vue";
import { ArrowPathIcon, ExclamationTriangleIcon } from "@heroicons/vue/24/outline";

const gameState = useGameStateStore();
const apiLoading = ref(false);

async function handleNewMove(move: number) {
	if (apiLoading.value) return;
	apiLoading.value = true;
	try {
		await gameState.submitMove(move);
	} catch (err) {
		console.error(err);
	}
	apiLoading.value = false;
}
</script>
