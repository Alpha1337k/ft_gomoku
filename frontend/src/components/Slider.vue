<template>
	<div>
		<label for="default-range" class="block mb-2 text-sm font-medium text-slate-900 dark:text-white">
			<slot>

			</slot>
		</label>
		<input id="default-range" v-model="modelValue" :max="max" :min="min" type="range" 
			class="w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer dark:bg-slate-700">
	</div>
</template>

<script setup lang="ts">

const modelValue = defineModel<number>({required: true, set(v: any) {
		return parseInt(v);
},});
const props = defineProps<{
	max: number;
	min: number;
}>()

</script>