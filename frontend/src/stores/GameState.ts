import { ref, computed, watchEffect, watch } from "vue";
import { defineStore } from "pinia";
import { ws } from "./api";

export interface Board {
	[key: number]: number | undefined;
}

export interface GameState {
	board: Board;
	currentTurn: number;
	score: number;
	moves: number[][];
	predictedMoves: number[];
	piecesTaken: number[];
}

export interface EvalState {
	boardScore: number
	moves: [{x: number, y: number}, number[]][]
}

export interface CalculationResponse {
	moves: {
		position: {
			x: number,
			y: number
		},
		order_idx: number,
		score: number
	}[];
	score: number;
}

export const useGameStateStore = defineStore("gameState", () => {
	const stateHistory = ref<GameState[]>([]);

	const currentState = ref<GameState>({
		score: 0,
		currentTurn: 0,
		board: {},
		moves: [],
		predictedMoves: [],
		piecesTaken: [0, 0]
	});

	const depth = ref(4);
	
	const isEditMode = ref(false);
	const editSettings = ref({
		is_maximizing: true
	})

	const editState = ref<Partial<EvalState>>();

	ws.emitter.on('boardUpdate', (b: {data: number[]}) => {
		let newBoard = {} as Board;

		console.log(b);

		for (let i = 0; i < b.data.length; i++) {
			if (b.data[i] == -1) continue;

			newBoard[i] = b.data[i]
		}

		console.log(newBoard);

		currentState.value.board = newBoard;
	})

	async function submitEdit() {
		const response = await ws.sendMessage<EvalState>("evaluate", {
			board: currentState.value.board,
			player: editSettings.value.is_maximizing ? 0 : 1,
		});

		if (response.boardScore == 1234) {
			response.boardScore = Infinity;
		} else if (response.boardScore == -1234) {
			response.boardScore = -Infinity;
		} else {
			currentState.value.score = response.boardScore;
		}

		editState.value = response;
	}

	async function submitMove(move: number) {
		let response = {} as CalculationResponse;

		const move_push = [move];
		response = await ws.sendMessage<CalculationResponse>("calculate", {
			depth: depth.value,
			board: currentState.value.board,
			turn_idx: currentState.value.currentTurn,
			in_move: {
				x: move % 19,
				y: Math.floor(move / 19)
			},
			player: 0,
		});
		console.log(response);
		const aiMove = response.moves.shift()!;
	
		if (aiMove) {
			move_push.push(aiMove.position.x + aiMove.position.y * 19);
			currentState.value.currentTurn = 0;
		}

		currentState.value.moves.push(move_push);

		if (response.score == 1234) {
			currentState.value.score = Infinity;
		} else if (response.score == -1234) {
			currentState.value.score = -Infinity;
		} else {
			currentState.value.score = response.score;
		}

		currentState.value.predictedMoves = response.moves;

		return response;
	}

	return { currentState, stateHistory, submitMove, isEditMode, depth, submitEdit, editState, editSettings };
});

export function getHumanPosition(pos: number) {
	const yName = String.fromCharCode(Math.floor(pos / 19) + 65);

	return yName + (pos % 19);
}
