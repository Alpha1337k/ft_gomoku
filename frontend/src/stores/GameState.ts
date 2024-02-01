import { ref, computed } from "vue";
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
}

export interface EvalState {
	boardScore: number
	evalPrio: number[]
}

export interface CalculationResponse {
	moves: number[];
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
	});

	const depth = ref(4);
	
	const isEditMode = ref(false);

	const editState = ref<Partial<EvalState>>();

	async function submitEdit() {
		const response = await ws.sendMessage<EvalState>("evaluate", {
			board: currentState.value.board
		});

		editState.value = response;
	}

	async function submitMove(move: number) {
		let response = {} as CalculationResponse;
		if (currentState.value.currentTurn == 1) {
			currentState.value.board[move] = 1;
			currentState.value.moves[currentState.value.moves.length - 1].push(move);
			response = await ws.sendMessage<CalculationResponse>("calculate", currentState.value);
			console.log(response);

			currentState.value.currentTurn = 0;
			console.log(response);
		}
		else {
			currentState.value.board[move] = 0;
			const move_push = [move];
	
			currentState.value.moves.push(move_push);
			currentState.value.currentTurn = 1;

	
			response = await ws.sendMessage<CalculationResponse>("calculate", currentState.value);
			console.log(response);
			const aiMove = response.moves.shift()!;
	
			if (aiMove) {
				move_push.push(aiMove);
				currentState.value.board[aiMove] = 1;
				currentState.value.currentTurn = 0;
			}
		}

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

	return { currentState, stateHistory, submitMove, isEditMode, depth, submitEdit, editState };
});

export function getHumanPosition(pos: number) {
	const yName = String.fromCharCode(Math.floor(pos / 19) + 65);

	return yName + (pos % 19);
}
