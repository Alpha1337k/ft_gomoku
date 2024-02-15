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
	const editSettings = ref({
		is_maximizing: true
	})

	const editState = ref<Partial<EvalState>>();

	async function submitEdit() {
		const response = await ws.sendMessage<EvalState>("evaluate", {
			board: currentState.value.board,
			is_maximizing: editSettings.value.is_maximizing
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

	function handleMove(move: number, is_maximizing: number) {
		let maps = [
			[move - 1, move - 2, move - 3],
			[move +  1, move + 2, move + 3],
			[move +  19, move + 38, move + 57],
			[move -  19, move - 38, move - 57],
			[move - 1 - 19, move - 2 - 38, move - 3 - 57],
			[move + 1 + 19, move + 2 + 38, move + 3 + 57],
			[move - 1 + 19, move - 2 + 38, move - 3 + 57],
			[move + 1 - 19, move + 2 - 38, move + 3 - 57],
		];


		for (let i = 0; i < maps.length; i++) {
			const e = maps[i];
			const oppVal = !is_maximizing ? 1 : 0;

			console.log(e);

			let oldPos = move;

			for (let n = 0; n < 3; n++) {
				if (e[n] >= 19 * 19 || 
					e[n] < 0 || 
					(oldPos % 19 == 18 && e[n] % 19 == 0) ||
					(oldPos % 19 == 0 && e[n] % 19 == 18)) {
						return;
				}
				oldPos = e[n];
			}

			if (
				currentState.value.board[e[0]] == oppVal &&
				currentState.value.board[e[1]] == oppVal &&
				currentState.value.board[e[2]] == is_maximizing
			) {
				currentState.value.board[e[0]] = undefined;
				currentState.value.board[e[1]] = undefined;
				console.log(e, oppVal, "SUCCESS");
			}
		}
	}

	async function submitMove(move: number) {
		let response = {} as CalculationResponse;
		if (currentState.value.currentTurn == 1) {
			currentState.value.board[move] = 1;
			handleMove(move, 1);
			currentState.value.moves[currentState.value.moves.length - 1].push(move);
			response = await ws.sendMessage<CalculationResponse>("calculate", {
				depth: depth.value,
				...currentState.value
			});
			console.log(response);

			currentState.value.currentTurn = 0;
			console.log(response);
		}
		else {
			currentState.value.board[move] = 0;
			handleMove(move, 0);
			
			const move_push = [move];
	
			currentState.value.moves.push(move_push);
			currentState.value.currentTurn = 1;

			response = await ws.sendMessage<CalculationResponse>("calculate", {
				depth: depth.value,
				...currentState.value
			});
			console.log(response);
			const aiMove = response.moves.shift()!;

	
			if (aiMove) {
				move_push.push(aiMove);
				currentState.value.board[aiMove] = 1;
				handleMove(aiMove, 1);
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

	return { currentState, stateHistory, submitMove, isEditMode, depth, submitEdit, editState, editSettings };
});

export function getHumanPosition(pos: number) {
	const yName = String.fromCharCode(Math.floor(pos / 19) + 65);

	return yName + (pos % 19);
}
