import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { ws } from "./api";

export interface Board {
	[key: number]: number;
}

export interface GameState {
	board: Board;
	currentTurn: number;
	score: number;
	moves: number[];
}

export const useGameStateStore = defineStore("gameState", () => {
	const stateHistory = ref<GameState[]>([]);

	const currentState = ref<GameState>({
		score: 0,
		currentTurn: 0,
		board: {},
		moves: [],
	});

	async function submitMove(move: number) {
		currentState.value.moves.push(move);

		const response = await ws.sendMessage<GameState>("calculate", currentState.value);

		currentState.value = response;

		return response;
	}

	return { currentState, stateHistory, submitMove };
});
