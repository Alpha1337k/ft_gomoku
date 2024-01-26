<template>
	<div class="grid grid-cols-19 grid-rows-19 size-[36rem] aspect-square">
		<div
			v-for="i in 361"
			:key="i"
			:class="[i % 2 == 0 ? 'bg-black' : 'bg-slate-800']"
			class="cursor-pointer flex items-center justify-center"
			@mouseover="hoverPos = i"
			@mouseleave="hoverPos = undefined"
			@click="handleClick(i)"
		>
			<div v-if="boardPositions[i] == 1" class="rounded-xl bg-blue-800 h-5/6 w-5/6"></div>
			<div v-else-if="boardPositions[i] == 2" class="rounded-xl bg-red-800 h-5/6 w-5/6"></div>
			<div v-else-if="hoverPos == i" class="rounded-xl bg-blue-800/75 h-5/6 w-5/6"></div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { Board } from "@/stores/GameState";

const hoverPos = ref<number>();

const props = defineProps<{
	boardPositions: Board;
	isLoading: boolean;
}>();

const emit = defineEmits(["moveChosen"]);

function handleClick(pos: number) {
	if (props.boardPositions[pos] == undefined) {
		emit("moveChosen", pos);
	}
}
</script>
