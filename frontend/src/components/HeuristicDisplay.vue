<template>
	<div>
		<p v-if="score === undefined" class="font-semibold">
			{{ placeholder }}
		</p>
		<h2 v-else-if="mate_in && mate_in < 0" class="bg-red-600 font-semibold min-w-10 max-w-20 w-fit rounded text-sm px-1 text-slate-300">
			M{{ -mate_in }}
		</h2>
		<h2 v-else-if="mate_in && mate_in > 0" class="bg-blue-600 font-semibold text-white min-w-10 max-w-20 w-fit rounded text-sm px-1">
			M{{ mate_in }}
		</h2>
		<h2 v-else-if="score < 0" class="bg-red-600 min-w-10 max-w-20 w-fit rounded text-sm px-1 text-slate-300">
			{{ score.toPrecision(3).substring(0, 4) }}
		</h2>
		<h2 v-else-if="score == 0" class="bg-slate-900 min-w-10 max-w-20 w-fit rounded text-sm px-1">
			{{ score.toPrecision(3).substring(0, 4) }}
		</h2>
		<h2 v-else-if="score > 0" class="bg-blue-600 text-white min-w-10 max-w-20 w-fit rounded text-sm px-1">
			+{{ score.toPrecision(3).substring(0, 4) }}
		</h2>
	</div>
</template>

<script setup lang="ts">
const props = defineProps<{
	score?: number;
	mate_in?: number;
	placeholder?: string;
}>();
</script>
