<template>
	<div class="flex justify-between space-x-4 my-4">
		<div class="w-min space-y-2">
			<PlayerBanner
				:active-turn="userLoading"
				:player-type="Piece.Min"
				:playerName="player == Piece.Min ? 'Player' : 'AI'"
				:captures="captures[1]"
			>
				<UserIcon v-if="player == Piece.Min" class="h-7 w-10"></UserIcon>
				<img v-else src="/robot1.png" class="h-10 rounded-full" />
			</PlayerBanner>
			<GoBoard
				:board-positions="gameBoard"
				:is-loading="false"
				:current-player="player"
				:invalid-moves="invalidMoves || []"
				:latest-move="moves[moves.length - 1]"
				:suggested-move="hint"
				:is-edit-mode="isEditMode"
				:edit-state="editState"
				@move-chosen="handleMoveSet"
				@editPosChange="submitEdit"
				class="border border-slate-800 rounded-lg"
			>
			</GoBoard>
			<PlayerBanner
				:active-turn="userLoading"
				:player-type="Piece.Max"
				:captures="captures[0]"
				:playerName="player == Piece.Max ? 'Player' : 'AI'"
			>
				<UserIcon v-if="player == Piece.Max" class="h-7 w-10"></UserIcon>
				<img v-else src="/robot1.png" class="h-10 rounded-full" />
			</PlayerBanner>
		</div>
		<SidePanel :moves="moves" :score="score" :mate_in="mate_in">
			<template #top>
				<FutureMoves :moves="futureMoves" />
			</template>

			<template #bottom>
				<div class="max-h-96">
					<div class="flex divide-x divide-slate-500">
						<button @click="setMode('play')" :class="{ 'bg-slate-700': !isEditMode }" class="hover:bg-slate-700 w-full cursor-pointer p-2">
							Play
						</button>
						<button @click="setMode('edit')" :class="{ 'bg-slate-700': isEditMode }" class="hover:bg-slate-700 w-full cursor-pointer p-2">
							Edit
						</button>
					</div>
					<div class="p-2 flex-1">
						<div>
							<p>Board d0 evaluation: {{ editSettings.score?.toFixed(4) ?? "?" }}</p>
						</div>
						<div class="space-y-1">
							<p>
								Blue captures: <input class="bg-slate-900 rounded text-center pl-3" type="number" min="0" max="5" v-model="captures[0]" />
							</p>
							<p>
								Red captures : <input class="bg-slate-900 rounded text-center pl-3" type="number" min="0" max="5" v-model="captures[1]" />
							</p>
						</div>
						<div class="flex justify-between">
							<p>View prio for blue?</p>
							<input type="checkbox" v-model="editSettings.is_maximizing" @change="submitEdit()" />
						</div>
					</div>

					<AppButton :disabled="hintLoading" @click="loadHint" class="bg-slate-900 transition rounded-t-none h-12 !text-base items-center">
						{{ hintLoading ? "loading.." : "Request hint" }}
					</AppButton>
				</div>
			</template>
		</SidePanel>
	</div>
	<ChooseSideModal @chosen="setPlayer" :open="modalDisplay == 'colorSelect'"></ChooseSideModal>
	<EndScreenModal
		@new-game="reloadGame"
		@close="modalDisplay = ''"
		:closable="true"
		:open="modalDisplay == 'max'"
		:player="Piece.Max"
		player-name="Blue"
	>
	</EndScreenModal>
	<EndScreenModal
		@new-game="reloadGame"
		@close="modalDisplay = ''"
		:closable="true"
		:open="modalDisplay == 'min'"
		:player="Piece.Min"
		player-name="Red"
	>
	</EndScreenModal>
</template>

<script setup lang="ts">
import GoBoard from "@/components/GoBoard.vue";
import PlayerBanner from "@/components/PlayerBanner.vue";
import SidePanel from "@/components/SidePanel.vue";
import EndScreenModal from "@/components/EndScreenModal.vue";
import {
	type Move,
	Piece,
	type Board,
	useGameStateStore,
	type HotseatResponse,
	type CalculationResponse,
	type EvalState,
	type FutureMove,
} from "@/stores/GameState";
import { UserIcon } from "@heroicons/vue/24/outline";
import { computed, ref } from "vue";
import AppButton from "@/components/AppButton.vue";
import ChooseSideModal from "@/components/ChooseSideModal.vue";
import FutureMoves from "@/components/FutureMoves.vue";

const gameBoard = ref<Board>({});
const score = ref(0);
const mate_in = ref<number>();
const player = ref<Piece>(Piece.Max);
const moves = ref<Move[]>([]);
const futureMoves = ref<FutureMove[]>([]);
const gameState = useGameStateStore();
const captures = ref<[number, number]>([0, 0]);
const invalidMoves = ref<number[]>();
const modalDisplay = ref<"" | "colorSelect" | "max" | "min">("colorSelect");
const hint = ref<number>();
const hintLoading = ref(false);
const aiLoading = ref(false);
const isEditMode = ref(false);
const editState = ref<EvalState>();
const editSettings = ref({
	is_maximizing: true,
	score: 0,
});

const userLoading = computed(() => {
	if (aiLoading.value == true) {
		return player.value == Piece.Max ? Piece.Min : Piece.Max;
	}
	return player.value == Piece.Max ? Piece.Max : Piece.Min;
});

function setPlayer(p: Piece) {
	player.value = p;
	modalDisplay.value = "";

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
	score.value = 0;
	mate_in.value = undefined;
}

async function loadHint() {
	hintLoading.value = true;

	const calculationResponse = await gameState.ws.sendMessage<CalculationResponse>("calculate", {
		board: gameBoard.value,
		depth: 5,
		captures: captures.value,
		player: isEditMode.value ? (editSettings.value.is_maximizing ? 0 : 1) : player.value,
		is_hint: true,
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

async function handleMoveSet(data?: { position: number; player?: number }) {
	hint.value = undefined;

	aiLoading.value = true;
	const timerStart = performance.now();
	const newState = await gameState.ws.sendMessage<CalculationResponse>("calculate", {
		board: gameBoard.value,
		in_move: data
			? {
					x: data.position % 19,
					y: Math.floor(data.position / 19),
				}
			: undefined,
		depth: 5,
		player: player.value == Piece.Max ? Piece.Max : Piece.Min,
		captures: captures.value,
	});
	const timerEnd = performance.now();
	aiLoading.value = false;

	futureMoves.value = newState.moves;

	if (player.value == Piece.Min) {
		if (newState.current_score == 1234) {
			modalDisplay.value = "max";
			return;
		} else if (newState.current_score == -1234) {
			modalDisplay.value = "min";
			return;
		}
	}

	const aiMove = newState.moves.shift()!;

	if (!aiMove) {
		console.error("No move from AI");
		return;
	}

	score.value = newState.score;
	mate_in.value = newState.mate_in;

	if (player.value == Piece.Max) {
		moves.value.push({
			"0": data?.position,
			"1": aiMove.position.x + aiMove.position.y * 19,
			responseTime: timerEnd - timerStart,
		});
	} else {
		if (data) moves.value[moves.value.length - 1][1] = data.position;

		moves.value.push({
			"0": aiMove.position.x + aiMove.position.y * 19,
			responseTime: timerEnd - timerStart,
		});
	}

	await loadInvalidMoves();

	if (player.value == Piece.Max) {
		if (newState.current_score == 1234) {
			modalDisplay.value = "max";
			return;
		} else if (newState.current_score == -1234) {
			modalDisplay.value = "min";
			return;
		}
	}
}

async function submitEdit(data?: { position: number; player?: number }) {
	if (data) {
		gameBoard.value[data.position] = data.player;
	}

	const response = await gameState.ws.sendMessage<EvalState>("evaluate", {
		board: gameBoard.value,
		player: editSettings.value.is_maximizing ? Piece.Max : Piece.Min,
	});

	if (response.boardScore == 1234) {
		response.boardScore = Infinity;
	} else if (response.boardScore == -1234) {
		response.boardScore = -Infinity;
	} else {
		editSettings.value.score = response.boardScore;
	}

	editState.value = response;
}

async function setMode(mode: "play" | "edit") {
	if (mode == "edit") {
		isEditMode.value = true;
		submitEdit();
	} else {
		isEditMode.value = false;
		loadInvalidMoves();
	}
}
</script>
