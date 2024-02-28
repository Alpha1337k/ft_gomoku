
<template>
	<div class="flex justify-between space-x-4 my-4">
		<div class="w-min space-y-2">
			<PlayerBanner
				:active-turn="currentPlayer"
				:player-type="Piece.Min"
				playerName="Red"
			>
				<UserIcon class="h-7 w-10"></UserIcon>
		</PlayerBanner>
			<GoBoard
				:board-positions="gameBoard"
				:is-loading="false"
				:current-player="currentPlayer"
				@move-chosen="handleMoveSet"
				class="border border-slate-800 rounded-lg"
			>
	
			</GoBoard>
			<PlayerBanner
				playerName="Blue"
				:active-turn="currentPlayer"
				:player-type="Piece.Max"
			>
			<UserIcon class="h-7 w-10"></UserIcon>
			</PlayerBanner>
		</div>
		<SidePanel
			:moves="moves"
		>

		</SidePanel>
	</div>

</template>

<script setup lang="ts">
import GoBoard from '@/components/GoBoard.vue';
import PlayerBanner from '@/components/PlayerBanner.vue'
import SidePanel from '@/components/SidePanel.vue';
import { Move, Piece, type Board } from '@/stores/GameState';
import { UserIcon } from '@heroicons/vue/24/outline';
import { ref } from 'vue';

const gameBoard = ref<Board>({});
const currentPlayer = ref<Piece>(Piece.Max);
const moves = ref<Move[]>([]);

function handleMoveSet(pos: number) {
	gameBoard.value[pos] = currentPlayer.value;

	if (currentPlayer.value == Piece.Max) {
		currentPlayer.value = Piece.Min;
	} else {
		currentPlayer.value = Piece.Max;
	}
}

</script>