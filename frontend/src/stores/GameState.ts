import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { api } from "./api";

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

		const response = await api.post<GameState>("/calculate", currentState.value).then((d) => d.data);

		currentState.value = response;

		return response;
	}

	return { currentState, stateHistory, submitMove };
});
