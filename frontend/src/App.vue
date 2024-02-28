<template>
	<div class="bg-black w-screen h-screen font-mono text-white">
		<div class="container max-w-7xl mx-auto p-8">
			<h1 class="text-2xl font-semibold">ft_gomoku</h1>
			<p class="text-sm">By okruitho</p>

			<RouterView />

			<!-- <div class="flex justify-between space-x-4 my-4">
				<GoBoard
					@move-chosen="(v) => handleNewMove(v)"
					:board-positions="gameState.currentState.board"
					:is-loading="false"
					class="border border-slate-800 rounded-lg"
				/>
				<SidePanel />
			</div> -->
		</div>
	</div>
</template>

<script setup lang="ts">
import GoBoard from "@/components/GoBoard.vue";
import SidePanel from "@/components/SidePanel.vue";
import { useGameStateStore } from "./stores/GameState";
import { ref } from "vue";

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
