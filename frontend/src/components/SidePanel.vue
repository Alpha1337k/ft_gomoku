<template>
	<div class="bg-slate-800 w-96 rounded-lg flex flex-col">
		<div class="bg-slate-900 p-4 rounded-t-lg border-slate-900 border-lg flex justify-between">
			<div>
				<h2 v-if="props.moves.length == 0" class="font-semibold">Move to get started</h2>
				<p v-if="score === undefined"></p>
				<h2 v-else-if="score < 0" class="bg-red-600 min-w-10 max-w-20 w-fit rounded text-sm px-1 text-slate-300">
					{{ score.toPrecision(3).substring(0, 4) }}
				</h2>
				<h2 v-else-if="score == 0" class="bg-slate-900 min-w-10 max-w-20 w-fit rounded text-sm px-1">
					{{ score.toPrecision(3).substring(0, 4) }}
				</h2>
				<h2 v-else-if="score > 0" class="bg-blue-600 text-white min-w-10 max-w-20 w-fit rounded text-sm px-1">
					+{{ score.toPrecision(3).substring(0, 4) }}
				</h2>
			</div>
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
		<!-- <div class="h-64 bg-slate-900 rounded-b-lg flex flex-col">
			<div class="flex divide-x divide-slate-500">
				<button
					@click="gameState.setMode('play')"
					:class="{ 'bg-slate-700': !gameState.isEditMode }"
					class="hover:bg-slate-700 w-full cursor-pointer p-2"
				>
					Play
				</button>
				<button
					@click="gameState.setMode('edit')"
					:class="{ 'bg-slate-700': gameState.isEditMode }"
					class="hover:bg-slate-700 w-full cursor-pointer p-2"
				>
					Edit
				</button>
			</div>
			<div class="p-2 flex-1">
				<Slider :max="6" :min="1" v-model="gameState.depth"> Depth ({{ gameState.depth }}) </Slider>
				<div>
					<p>Board d0 evaluation: {{ gameState.editState?.boardScore?.toFixed(4) ?? "?" }}</p>
				</div>
				<div class="flex justify-between">
					<p>View prio for blue?</p>
					<input type="checkbox" v-model="gameState.editSettings.is_maximizing" @change="gameState.submitEdit()" />
				</div>
			</div>
			<button
				@click="
					gameState.currentState.board = {};
					gameState.submitEdit();
				"
				:class="{ 'bg-slate-700': !gameState.isEditMode }"
				class="hover:bg-slate-700 w-full bg-slate-700 cursor-pointer p-2"
			>
				Reset
			</button>
		</div> -->
	</div>
</template>

<script setup lang="ts">
import { getHumanPosition, useGameStateStore, type Move } from "@/stores/GameState";
import Slider from "@/components/Slider.vue";

const props = defineProps<{
	moves: Move[];
	score?: number;
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
