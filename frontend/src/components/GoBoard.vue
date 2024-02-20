<template>
	<div>
		<div class="grid ml-6 grid-cols-19">
			<div v-for="i in 19" :key="i" class="text-center">
				{{ i - 1 }}
			</div>
		</div>
		<div class="flex">
			<div class="grid grid-rows-19">
				<div v-for="i in 19" :key="i" class="my-auto w-6 text-center">
					{{ String.fromCharCode(i + 64) }}
				</div>
			</div>
			<div class="grid grid-cols-19 grid-rows-19 size-[56rem] aspect-square col-span-4" style="column-span: span 19 / span 19">
				<div
					v-for="i in 361"
					:key="i"
					:class="[i % 2 == 0 ? 'bg-black' : 'bg-slate-800']"
					class="cursor-pointer flex items-center justify-center"
					@mouseover="hoverPos = i - 1"
					@mouseleave="hoverPos = undefined"
					@click="handleClick($event, i - 1)"
					@contextmenu.prevent="handleRightClick(i - 1)"
				>
					<div v-if="boardPositions[i - 1] === 0" class="rounded-xl bg-blue-800 h-5/6 w-5/6"></div>
					<div v-else-if="boardPositions[i - 1] === 1" class="rounded-xl bg-red-800 h-5/6 w-5/6"></div>
					<div v-else-if="hoverPos == i - 1 && ctrlPressed == false" class="rounded-xl bg-blue-800/75 h-5/6 w-5/6"></div>
					<div v-else-if="hoverPos == i - 1 && ctrlPressed == true" class="rounded-xl bg-red-800/75 h-5/6 w-5/6"></div>
					<div v-else="evalPrioMap[i -1 ] != undefined" class="h-5/6 w-5/6">
							<p class="text-white h-10 mx-auto text-center">{{ evalPrioMap[i -1 ]?.idx }}</p>
							<p class="h-10 mx-auto text-center absolute text-sm text-gray-400 -mt-5 ml-1">{{ evalPrioMap[i -1 ]?.score.toFixed(2) }}</p>
					</div>			
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref } from "vue";
import { useGameStateStore, type Board } from "@/stores/GameState";

const hoverPos = ref<number>();

const props = defineProps<{
	boardPositions: Board;
	isLoading: boolean;
}>();

const gameState = useGameStateStore();

const emit = defineEmits(["moveChosen"]);
const ctrlPressed = ref(false);

const evalPrioMap = computed(() => {
	const mapped: {[key: number]: {
		idx: number,
		score: number
	}} = {}

	if (gameState.isEditMode && gameState.editState?.moves) {
		for (let i = 0; i < gameState.editState.moves.length; i++) {
			const move = gameState.editState.moves[i];
			mapped[move[0].x + move[0].y * 19] = {
				idx: i,
				score: move[1][0]
			};
		}
	}
	return mapped;
})

function updateAlt(e: KeyboardEvent) {
	if (e.ctrlKey && gameState.isEditMode) {
		ctrlPressed.value = true;
	} else {
		ctrlPressed.value = false;
	}
}

document.addEventListener("keydown", updateAlt);
document.addEventListener("keyup", updateAlt);

onUnmounted(() => {
	document.removeEventListener("keydown", updateAlt)
	document.removeEventListener("keyup", updateAlt)
});

function handleRightClick(pos: number) {
	if (gameState.isEditMode) {
		props.boardPositions[pos] = undefined;
		gameState.submitEdit();
	}
}

function handleClick(event: PointerEvent, pos: number) {
	
	if (gameState.isEditMode) {
		if (event.ctrlKey) {
			props.boardPositions[pos] = 1;
		} else {
			props.boardPositions[pos] = 0;
		}

		gameState.submitEdit();
	}
	
	if (props.boardPositions[pos] != undefined) {
		return;
	}
	
	if (gameState.isEditMode == false) {
		emit("moveChosen", pos);
	}
}
</script>
