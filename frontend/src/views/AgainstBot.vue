<template>
	<div class="flex justify-between space-x-4 my-4">
		<div class="w-min space-y-2">
			<PlayerBanner :active-turn="userLoading" :player-type="Piece.Min" 
				:playerName="player == Piece.Min ? 'Player' : 'AI'" :captures="captures[1]">
				<UserIcon v-if="player == Piece.Min" class="h-7 w-10"></UserIcon>
				<img v-else src="/robot1.png" class="h-10 rounded-full"/>
			</PlayerBanner>
			<GoBoard
				:board-positions="gameBoard"
				:is-loading="false"
				:current-player="player"
				:invalid-moves="invalidMoves || []"
				:suggested-move="hint"
				@move-chosen="handleMoveSet"
				class="border border-slate-800 rounded-lg"
			>
			</GoBoard>
			<PlayerBanner :active-turn="userLoading" :player-type="Piece.Max" :captures="captures[0]"
				:playerName="player == Piece.Max ? 'Player' : 'AI'">
					<UserIcon v-if="player == Piece.Max" class="h-7 w-10"></UserIcon>
					<img v-else src="/robot1.png" class="h-10 rounded-full"/>
			</PlayerBanner>
		</div>
		<SidePanel 
			:moves="moves"
			:score="score"
		>
		
			<template #bottom>
				<AppButton :disabled="hintLoading" @click="loadHint" class="bg-slate-900 transition rounded-t-none h-12 !text-base items-center">
					{{ hintLoading ? 'loading..' : 'Request hint' }} 
				</AppButton>			
			</template>
		</SidePanel>
	</div>
	<ChooseSideModal @chosen="setPlayer" :open="modalDisplay == 'colorSelect'"></ChooseSideModal>
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
import { computed, ref } from "vue";
import AppButton from "@/components/AppButton.vue";
import ChooseSideModal from "@/components/ChooseSideModal.vue";

const gameBoard = ref<Board>({});
const score = ref(0);
const player = ref<Piece>(Piece.Max);
const moves = ref<Move[]>([]);
const gameState = useGameStateStore();
const captures = ref<[number, number]>([0, 0]);
const invalidMoves = ref<number[]>();
const modalDisplay = ref<"" | "colorSelect" | "max" | "min">('colorSelect');
const hint = ref<number>();
const hintLoading = ref(false);
const aiLoading = ref(false);
const userLoading = computed(() => {
	if (aiLoading.value == true) {
		return player.value == Piece.Max ? Piece.Min : Piece.Max;
	}
	return player.value == Piece.Max ? Piece.Max : Piece.Min;
})


function setPlayer(p: Piece) {
	player.value = p;
	modalDisplay.value = ''

	if (p == Piece.Min) {
		handleMoveSet();
	}
}

function reloadGame() {
	gameBoard.value = {};
	player.value = Piece.Max;
	captures.value = [0, 0];
	invalidMoves.value = [];
	moves.value = [];
	modalDisplay.value = "";
}

async function loadHint() {
	hintLoading.value = true;

	const calculationResponse = await gameState.ws.sendMessage<CalculationResponse>('calculate', {
		board: gameBoard.value,
		depth: 5,
		captures: captures.value,
		player: player.value,
	});

	hintLoading.value = false;

	const move = calculationResponse.moves.shift()!;

	hint.value = move.position.x + move.position.y * 19;
}

async function loadInvalidMoves() {
	invalidMoves.value = undefined;
	const moves: { x: number; y: number }[] = await gameState.ws.sendMessage("inv_moves", {
		board: gameBoard.value,
		player: player.value,
	});

	invalidMoves.value = moves.map((x) => x.x + x.y * 19);
}


gameState.ws.emitter.on("boardUpdate", (b: any) => {
	if (aiLoading.value) {
		gameBoard.value = gameState.parseBoard(b.board.data);
		captures.value = b.captures;
	}
});

async function handleMoveSet(pos?: number) {
	hint.value = undefined;

	aiLoading.value = true;
	const timerStart = performance.now();
	const newState = await gameState.ws.sendMessage<CalculationResponse>("calculate", {
		board: gameBoard.value,
		in_move: pos ? {
			x: pos % 19,
			y: Math.floor(pos / 19),
		} : undefined,
		depth: 5,
		player: player.value == Piece.Max ? Piece.Max : Piece.Min,
		captures: captures.value,
	});
	const timerEnd = performance.now();
	aiLoading.value = false;

	const aiMove = newState.moves.shift()!;

	score.value = newState.score;

	if (player.value == Piece.Max) {
		moves.value.push({
			"0": pos,
			"1": aiMove.position.x + aiMove.position.y * 19,
			responseTime: timerEnd - timerStart
		});
	} else {
		if (pos)
			moves.value[moves.value.length - 1][1] = pos;

		moves.value.push({
			"0": aiMove.position.x + aiMove.position.y * 19,
			responseTime: timerEnd - timerStart
		});
	}

	await loadInvalidMoves();

	if (newState.score == 1234) {
		modalDisplay.value = "max";
	} else if (newState.score == -1234) {
		modalDisplay.value = "min";
	}
}

</script>