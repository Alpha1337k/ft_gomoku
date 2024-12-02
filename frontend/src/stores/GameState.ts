import { ref, computed, watchEffect, watch } from "vue";
import { defineStore } from "pinia";
import { WebSocketAPI } from "./api";

export interface Board {
	[key: number]: number | undefined;
}

export enum Piece {
	Max = 0,
	Min = 1,
}

export interface FutureMove {
	cutoff_at: number;
	order_idx: number;
	position: { x: number; y: number };
}

export interface GameState {
	board: Board;
	currentTurn: number;
	score: number;
	moves: number[][];
	predictedMoves: FutureMove[];
	captures: number[];
	mate_in?: number
}

export type Move = { 0?: number; 1?: number; responseTime?: number | null; order_idx?: number };

export interface EvalState {
	boardScore: number;
	moves: [{ x: number; y: number }, number[]][];
}

export interface BoardUpdateResponse {
	board: {
		data: number[];
	};
	captures: number[];
}

export interface HotseatResponse {
	board: {
		data: number[];
	};
	captures: [number, number];
	score: number;
}

export interface CalculationResponse {
	moves: FutureMove[];
	current_score: number;
	score: number;
	mate_in: number;
}

export const useGameStateStore = defineStore("gameState", () => {
	const stateHistory = ref<GameState[]>([]);

	const ws = ref(new WebSocketAPI());
	const wsOK = ref<boolean | undefined>(undefined);

	ws.value.initWebsocket((v) => {
		wsOK.value = v;
	});

	const moveHistory = ref<Move[]>([]);

	const currentState = ref<GameState>({
		score: 0,
		currentTurn: 0,
		board: {},
		moves: [],
		predictedMoves: [],
		captures: [0, 0],
	});

	const invalidMoves = ref<number[]>();

	async function loadInvalidMoves() {
		invalidMoves.value = undefined;
		const moves: { x: number; y: number }[] = await ws.value.sendMessage("inv_moves", {
			board: currentState.value.board,
			player: 0,
		});

		invalidMoves.value = moves.map((x) => x.x + x.y * 19);
	}

	const depth = ref(4);

	const isEditMode = ref(false);
	const editSettings = ref({
		is_maximizing: true,
	});

	const editState = ref<Partial<EvalState>>();

	function parseBoard(b: number[]): Board {
		const newBoard = {} as Board;

		for (let i = 0; i < b.length; i++) {
			if (b[i] == -1) continue;

			newBoard[i] = b[i];
		}

		return newBoard;
	}

	ws.value.emitter.on("boardUpdate", (b: BoardUpdateResponse) => {
		currentState.value.board = parseBoard(b.board.data);
		currentState.value.captures = b.captures;
	});

	ws.value.emitter.on("ready", () => {
		loadInvalidMoves();
	});

	async function submitEdit() {
		const response = await ws.value.sendMessage<EvalState>("evaluate", {
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

		const newMove = {
			0: move,
			1: undefined,
			responseTime: undefined,
		} as any;

		moveHistory.value.push(newMove);

		const move_push = [move];

		const timerStart = performance.now();
		response = await ws.value.sendMessage<CalculationResponse>("calculate", {
			depth: depth.value,
			board: currentState.value.board,
			in_move: {
				x: move % 19,
				y: Math.floor(move / 19),
			},
			player: 0,
			captures: currentState.value.captures,
		});
		const timerEnd = performance.now();

		console.log(response);
		response.moves.pop();
		const aiMove = response.moves[0]!;

		console.log("ORDER_IDX_FIRST", aiMove.order_idx);

		if (aiMove) {
			move_push.push(aiMove.position.x + aiMove.position.y * 19);
			currentState.value.currentTurn = 0;
		}

		currentState.value.moves.push(move_push);
		newMove[1] = move_push[1];
		newMove.responseTime = timerEnd - timerStart;

		currentState.value.score = response.score;
		currentState.value.mate_in = response.mate_in;

		currentState.value.predictedMoves = response.moves;

		loadInvalidMoves();

		return response;
	}

	function setMode(mode: "play" | "edit") {
		if (mode == "edit") {
			isEditMode.value = true;
			submitEdit();
		} else {
			isEditMode.value = false;
			loadInvalidMoves();
		}
	}

	return {
		currentState,
		stateHistory,
		setMode,
		invalidMoves,
		submitMove,
		isEditMode,
		depth,
		submitEdit,
		editState,
		editSettings,
		moveHistory,
		parseBoard,
		ws,
		wsOK,
	};
});

export function getHumanPosition(pos: number) {
	const yName = String.fromCharCode(Math.floor(pos / 19) + 65);

	return yName + (pos % 19);
}
