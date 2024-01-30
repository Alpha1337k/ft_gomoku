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
					@click="handleClick(i - 1)"
				>
					<div v-if="boardPositions[i - 1] === 0" class="rounded-xl bg-blue-800 h-5/6 w-5/6"></div>
					<div v-else-if="boardPositions[i - 1] === 1" class="rounded-xl bg-red-800 h-5/6 w-5/6"></div>
					<div v-else-if="hoverPos == i - 1" class="rounded-xl bg-blue-800/75 h-5/6 w-5/6"></div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useGameStateStore, type Board } from "@/stores/GameState";

const hoverPos = ref<number>();

const props = defineProps<{
	boardPositions: Board;
	isLoading: boolean;
}>();

const gameState = useGameStateStore();

const emit = defineEmits(["moveChosen"]);

function handleClick(pos: number) {
	if (props.boardPositions[pos] == undefined && gameState.isEditMode == false) {
		emit("moveChosen", pos);
	}
}
</script>
