<template>
	<div :class="['w-full flex space-x-4 items-center border-lg h-12 rounded-lg p-2 border border-white transition', bannerColors]">
		<slot>

		</slot>
		<p>
			{{ playerName }}
		</p>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Piece } from '@/stores/GameState';

const bannerColors = computed(() => {
	if (props.playerType == Piece.Max && props.playerType == props.activeTurn) {
		return "bg-blue-600 border-white"
	} else if (props.playerType == Piece.Min && props.playerType == props.activeTurn) {
		return "bg-red-600 border-white"
	}
	return "bg-black"
})

const props = defineProps<{
	playerName: string,
	activeTurn: Piece,
	playerType: Piece
}>();

</script>