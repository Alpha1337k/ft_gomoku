<template>
	<div class="bg-slate-800 w-96 rounded-lg flex flex-col">
		<div class="bg-slate-900 p-4 rounded-t-lg border-slate-900 border-lg flex justify-between">
			<HeuristicDisplay :mate_in="mate_in" :score="score" placeholder="Move to get started." />
			<div class="flex-1 mx-2 text-sm" v-if="gameState.currentState.predictedMoves[0]">
				<p :class="getIdxColor(gameState.currentState.predictedMoves[0].order_idx)">
					{{ gameState.currentState.predictedMoves[0].order_idx }}
				</p>
			</div>
			<div v-if="gameState.currentState.moves.length > 0" class="divide-x divide-slate-300 flex">
				<p class="px-2 text-blue-300">{{ gameState.currentState.captures[0] }}</p>
				<p class="px-2 text-red-400">{{ gameState.currentState.captures[1] }}</p>
			</div>
		</div>
		<slot name="top"> </slot>
		<div class="space-y-2 my-2 flex-1 overflow-y-auto">
			<div v-for="(move, i) in moves" :key="i" class="flex justify-between text-sm px-4 text-slate-200">
				<div class="flex justify-start">
					<p class="text-slate-500 w-6">{{ i }}.</p>
					<p
						class="w-10 px-1"
						v-if="move[0] != undefined"
						:class="move[1] == undefined ? 'bg-slate-600 rounded border-b-4 border-slate-500' : ''"
					>
						{{ getHumanPosition(move[0]) }}
					</p>
					<p
						class="w-10 px-1"
						v-if="move[1] != undefined"
						:class="i == moves.length - 1 ? 'bg-slate-600 rounded border-b-4 border-slate-500' : ''"
					>
						{{ getHumanPosition(move[1]) }}
					</p>
				</div>
				<p class="text-xs text-slate-600">{{ move.responseTime?.toFixed(0) || "?" }} ms</p>
			</div>
		</div>

		<slot name="bottom"> </slot>
	</div>
</template>

<script setup lang="ts">
import { getHumanPosition, useGameStateStore, type Move } from "@/stores/GameState";
import HeuristicDisplay from "./HeuristicDisplay.vue";

const props = defineProps<{
	moves: Move[];
	score?: number;
	mate_in?: number;
}>();

const gameState = useGameStateStore();

function getIdxColor(idx: number) {
	if (idx < 5) {
		return "text-white";
	} else if (idx < 10) {
		return "text-green-500";
	} else if (idx < 20) {
		return "text-amber-500";
	} else {
		return "text-red-500";
	}
}
</script>
