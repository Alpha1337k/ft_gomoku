<template>
	<div class="flex bg-slate-950 px-2 space-x-2 text-slate-400 text-sm">
		<div :key="i" v-for="(move, i) in moves" class="flex">
			<p :class="getIdxColor(move.order_idx)">
				{{ getHumanPosition(move.position.x + move.position.y * 19) }}
			</p>
			<div class="text-slate-800 mx-2" v-if="i % 2 == 1">|</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { getHumanPosition, type FutureMove, type Move } from "@/stores/GameState";

const props = defineProps<{
	moves: FutureMove[];
}>();
const emit = defineEmits([]);

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
