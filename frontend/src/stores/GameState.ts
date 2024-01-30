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

	async function submitMove(move: number) {
		currentState.value.board[move] = 0;
		const move_push = [move];

		currentState.value.moves.push(move_push);

		const response = await ws.sendMessage<CalculationResponse>("calculate", currentState.value);
		const aiMove = response.moves.pop()!;

		currentState.value.board[aiMove] = 1;

		currentState.value.score = response.score;
		currentState.value.predictedMoves = response.moves;
		move_push.push(aiMove);

		return response;
	}

	return { currentState, stateHistory, submitMove };
});

export function getHumanPosition(pos: number) {
	const yName = String.fromCharCode(Math.floor(pos / 19) + 65);

	return yName + (pos % 19);
}
