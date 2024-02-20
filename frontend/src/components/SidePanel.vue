<template>
	<div class="bg-slate-800 w-96 rounded-lg flex flex-col">
		<div class="bg-slate-900 p-4 rounded-t-lg border-slate-900 border-lg">
			<h2 v-if="gameState.currentState.moves.length == 0" class="font-semibold">Move to get started</h2>
			<h2 v-else-if="gameState.currentState.score < 0" class="bg-slate-950 min-w-10 max-w-20 w-fit rounded text-sm px-1 text-slate-300">
				{{ gameState.currentState.score.toPrecision(3).substring(0, 4) }}
			</h2>
			<h2 v-else-if="gameState.currentState.score == 0" class="bg-slate-900 min-w-10 max-w-20 w-fit rounded text-sm px-1">
				{{ gameState.currentState.score.toPrecision(3).substring(0, 4) }}
			</h2>
			<h2 v-else-if="gameState.currentState.score > 0" class="bg-slate-300 text-black min-w-10 max-w-20 w-fit rounded text-sm px-1">
				+{{ gameState.currentState.score.toPrecision(3).substring(0, 4) }}
			</h2>
		</div>
		<div class="flex bg-slate-950 px-2 space-x-2 text-slate-400 text-sm">
			<div :key="i" v-for="(move, i) in gameState.currentState.predictedMoves" class="flex">
				<p>
					{{ getHumanPosition(move) }}
				</p>
				<div class="text-slate-800 mx-2" v-if="i % 2 == 1">|</div>
			</div>
		</div>
		<div class="space-y-2 my-2 flex-1">
			<div v-for="(moves, i) in gameState.currentState.moves" :key="i" class="flex justify-start text-sm px-4 text-slate-200">
				<p class="text-slate-500 w-6">{{ i }}.</p>
				<p class="w-10 px-1" :class="moves[1] == undefined ? 'bg-slate-600 rounded border-b-4 border-slate-500' : ''">
					{{ getHumanPosition(moves[0]) }}
				</p>
				<p
					class="w-10 px-1"
					v-if="moves[1] != undefined"
					:class="i == gameState.currentState.moves.length - 1 ? 'bg-slate-600 rounded border-b-4 border-slate-500' : ''"
				>
					{{ getHumanPosition(moves[1]) }}
				</p>
			</div>
		</div>
		<div class="h-64 bg-slate-900 rounded-b-lg flex flex-col">
			<div class="flex divide-x divide-slate-500">
				<button @click="gameState.isEditMode = false; gameState.editState = {}" :class="{'bg-slate-700': !gameState.isEditMode}" class="hover:bg-slate-700 w-full  cursor-pointer p-2">
					Play
				</button>
				<button @click="gameState.isEditMode = true; gameState.submitEdit()" :class="{'bg-slate-700': gameState.isEditMode}" class="hover:bg-slate-700 w-full  cursor-pointer p-2">
					Edit
				</button>
			</div>
			<div class="p-2 flex-1">
				<Slider :max="6" :min="1" v-model="gameState.depth">
					Depth ({{ gameState.depth }})
				</Slider>
				<div>
					<p>Board d0 evaluation: {{ gameState.editState?.boardScore?.toFixed(4) ?? "?" }}</p>
				</div>
				<div class="flex justify-between">
					<p>View prio for blue?</p>
					<input type="checkbox" v-model="gameState.editSettings.is_maximizing" @change="gameState.submitEdit()"/>
				</div>
			</div>
			<button @click="gameState.currentState.board = {}; gameState.submitEdit()" :class="{'bg-slate-700': !gameState.isEditMode}" class="hover:bg-slate-700 w-full bg-slate-700  cursor-pointer p-2">
					Reset
			</button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { getHumanPosition, useGameStateStore } from "@/stores/GameState";
import Slider from "@/components/Slider.vue";

const gameState = useGameStateStore();

console.log(gameState.currentState.score);
</script>
