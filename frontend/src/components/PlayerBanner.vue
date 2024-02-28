<template>
	<div :class="['w-full flex justify-between items-center border-lg h-12 rounded-lg p-2 border border-white transition', bannerColors]">
		<div class="flex space-x-4 items-center">
			<slot> </slot>
			<p>
				{{ playerName }}
			</p>
		</div>
		<div class="bg-slate-200 rounded-lg w-6 h-8 border">
			<p class="text-slate-800 text-center mt-1">{{ captures.toString() }}</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Piece } from "@/stores/GameState";

const bannerColors = computed(() => {
	if (props.playerType == Piece.Max && props.playerType == props.activeTurn) {
		return "bg-blue-600 border-white";
	} else if (props.playerType == Piece.Min && props.playerType == props.activeTurn) {
		return "bg-red-600 border-white";
	}
	return "bg-black";
});

const props = defineProps<{
	playerName: string;
	activeTurn: Piece;
	playerType: Piece;
	captures: number;
}>();
</script>
