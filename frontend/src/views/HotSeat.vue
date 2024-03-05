<template>
	<div class="flex justify-between space-x-4 my-4">
		<div class="w-min space-y-2">
			<PlayerBanner :active-turn="currentPlayer" :player-type="Piece.Min" playerName="Red" :captures="captures[1]">
				<UserIcon class="h-7 w-10"></UserIcon>
			</PlayerBanner>
			<GoBoard
				:board-positions="gameBoard"
				:is-loading="false"
				:current-player="currentPlayer"
				:invalid-moves="invalidMoves || []"
				:suggested-move="hint"
				@move-chosen="handleMoveSet"
				class="border border-slate-800 rounded-lg"
			>
			</GoBoard>
			<PlayerBanner playerName="Blue" :active-turn="currentPlayer" :player-type="Piece.Max" :captures="captures[0]">
				<UserIcon class="h-7 w-10"></UserIcon>
			</PlayerBanner>
		</div>
		<SidePanel :moves="moves">
			<template #bottom>
				<AppButton :disabled="hintLoading" @click="loadHint" class="bg-slate-900 transition rounded-t-none h-12 !text-base items-center">
					{{ hintLoading ? "loading.." : "Request hint" }}
				</AppButton>
			</template>
		</SidePanel>
	</div>
	<EndScreenModal @new-game="reloadGame" :open="modalDisplay == 'max'" :player="Piece.Max" player-name="Blue"> </EndScreenModal>
	<EndScreenModal @new-game="reloadGame" :open="modalDisplay == 'min'" :player="Piece.Min" player-name="Red"> </EndScreenModal>
</template>

<script setup lang="ts">
import GoBoard from "@/components/GoBoard.vue";
import PlayerBanner from "@/components/PlayerBanner.vue";
import SidePanel from "@/components/SidePanel.vue";
import EndScreenModal from "@/components/EndScreenModal.vue";
import { type Move, Piece, type Board, useGameStateStore, type HotseatResponse, type CalculationResponse } from "@/stores/GameState";
import { UserIcon } from "@heroicons/vue/24/outline";
import { ref } from "vue";
import AppButton from "@/components/AppButton.vue";

const gameBoard = ref<Board>({});
const currentPlayer = ref<Piece>(Piece.Max);
const moves = ref<Move[]>([]);
const gameState = useGameStateStore();
const captures = ref<[number, number]>([0, 0]);
const invalidMoves = ref<number[]>();
const modalDisplay = ref<"" | "max" | "min">();
const hint = ref<number>();
const hintLoading = ref(false);

function reloadGame() {
	gameBoard.value = {};
	(currentPlayer.value = Piece.Max), (moves.value = []);
	captures.value = [0, 0];
	invalidMoves.value = [];
	modalDisplay.value = "";
}

async function loadHint() {
	hintLoading.value = true;

	const calculationResponse = await gameState.ws.sendMessage<CalculationResponse>("calculate", {
		board: gameBoard.value,
		depth: 5,
		captures: captures.value,
		player: currentPlayer.value,
	});

	hintLoading.value = false;

	const move = calculationResponse.moves.shift()!;

	hint.value = move.position.x + move.position.y * 19;
}

async function loadInvalidMoves() {
	invalidMoves.value = undefined;
	const moves: { x: number; y: number }[] = await gameState.ws.sendMessage("inv_moves", {
		board: gameBoard.value,
		player: currentPlayer.value,
	});

	invalidMoves.value = moves.map((x) => x.x + x.y * 19);
}

async function handleMoveSet(pos: number) {
	hint.value = undefined;

	const newState = await gameState.ws.sendMessage<HotseatResponse>("hotseat_move", {
		board: gameBoard.value,
		in_move: {
			x: pos % 19,
			y: Math.floor(pos / 19),
		},
		player: currentPlayer.value,
		captures: captures.value,
	});

	if (currentPlayer.value == Piece.Max) {
		moves.value.push({
			"0": pos,
		});
		currentPlayer.value = Piece.Min;
	} else {
		moves.value[moves.value.length - 1][1] = pos;

		currentPlayer.value = Piece.Max;
	}

	gameBoard.value = gameState.parseBoard(newState.board.data);

	await loadInvalidMoves();

	captures.value = newState.captures;
	if (newState.score == 1234) {
		modalDisplay.value = "max";
	} else if (newState.score == -1234) {
		modalDisplay.value = "min";
	}
}
</script>
