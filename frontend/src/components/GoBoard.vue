<template>
	<div>
		<div class="grid ml-6 grid-cols-19 w-[56rem]">
			<div v-for="i in 19" :key="i" class="text-center">
				{{ i - 1 }}
			</div>
		</div>
		<div class="flex w-[57.5rem]">
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
					class="cursor-pointer flex items-center justify-center h-auto"
					@mouseover="hoverPos = i - 1"
					@mouseleave="hoverPos = undefined"
					@click="handleClick($event as any, i - 1)"
					@contextmenu.prevent="handleRightClick(i - 1)"
				>
					<div v-if="evalPrioMap[i - 1] == undefined || hoverPos == i - 1" :class="getColor(i - 1)" class="rounded-xl h-5/6 w-5/6"></div>
					<div v-else-if="evalPrioMap[i - 1]" class="h-10 absolute flex flex-col items-center">
						<p class="text-white mx-auto text-center">{{ evalPrioMap[i - 1]?.idx }}</p>
						<p class="mx-auto text-center text-sm text-gray-400">{{ resolveScore(evalPrioMap[i - 1]?.score) }}</p>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref } from "vue";
import { Piece, type Board, type EvalState } from "@/stores/GameState";

const hoverPos = ref<number>();

const props = defineProps<{
	boardPositions: Board;
	currentPlayer: Piece;
	suggestedMove?: number;
	isLoading: boolean;
	isEditMode?: boolean;
	editState?: EvalState;
	latestMove?: { 0?: number; 1?: number };
	invalidMoves: number[];
	suggestedPosition?: {
		player: Piece;
		pos: number;
	};
}>();

const emit = defineEmits(["moveChosen", "editPosChange"]);
const ctrlPressed = ref(false);

const evalPrioMap = computed(() => {
	const mapped: {
		[key: number]: {
			idx: number;
			score: number;
		};
	} = {};

	if (props.isEditMode && props.editState?.moves) {
		for (let i = 0; i < props.editState.moves.length; i++) {
			const move = props.editState.moves[i];
			mapped[move[0].x + move[0].y * 19] = {
				idx: i,
				score: move[1][0],
			};
		}
	}
	return mapped;
});

function updateCtrl(e: KeyboardEvent) {
	if (e.ctrlKey && props.isEditMode) {
		ctrlPressed.value = true;
	} else {
		ctrlPressed.value = false;
	}
}

document.addEventListener("keydown", updateCtrl);
document.addEventListener("keyup", updateCtrl);

onUnmounted(() => {
	document.removeEventListener("keydown", updateCtrl);
	document.removeEventListener("keyup", updateCtrl);
});

function handleRightClick(pos: number) {
	if (props.isEditMode) {
		emit("editPosChange", { position: pos, player: undefined });
	}
}

function resolveScore(v: number): string {
	if (v === undefined || v === null) {
		return "";
	}

	return v.toPrecision(3).substring(0, 4);
}

function getHoverPlayer(): Piece {
	if (props.isEditMode) {
		if (ctrlPressed.value == true) {
			return Piece.Min;
		}
		return Piece.Max;
	}
	return props.currentPlayer;
}

function getColor(pos: number) {
	if (props.invalidMoves?.find((x) => x == pos)) {
		return "bg-yellow-500";
	}

	if (pos == props.suggestedMove) {
		return "bg-orange-500";
	}

	if (props.boardPositions[pos] === 0) {
		if (props.latestMove?.[0] == pos) {
			return "bg-blue-500";
		} else {
			return "bg-blue-800";
		}
	}
	if (props.boardPositions[pos] === 1) {
		if (props.latestMove?.[1] == pos) {
			return "bg-red-700";
		} else {
			return "bg-red-800";
		}
	}

	const player = getHoverPlayer();

	if (hoverPos.value == pos && player == Piece.Max) {
		return "bg-blue-800/75";
	}
	if (hoverPos.value == pos && player == Piece.Min) {
		return "bg-red-800/75";
	}

	if (props.suggestedPosition && props.suggestedPosition.pos == pos) {
		if (props.suggestedPosition.player == Piece.Max) {
			return "bg-blue-800/75";
		}
		return "bg-red-800/75";
	}
}

function handleClick(event: PointerEvent, pos: number) {
	let player = undefined;

	if (props.isEditMode) {
		if (event.ctrlKey) {
			player = 1;
		} else {
			player = 0;
		}
	}

	if (props.boardPositions[pos] != undefined || props.invalidMoves.find((x) => x == pos)) {
		return;
	}

	if (props.isEditMode == false) {
		emit("moveChosen", { position: pos, player });
	} else {
		emit("editPosChange", { position: pos, player });
	}
}
</script>
