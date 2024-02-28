<template>
	<TransitionRoot as="template" :show="open">
		<Dialog as="div" class="relative z-10">
			<TransitionChild
				as="template"
				enter="ease-out duration-300"
				enter-from="opacity-0"
				enter-to="opacity-100"
				leave="ease-in duration-200"
				leave-from="opacity-100"
				leave-to="opacity-0"
			>
				<div class="fixed inset-0 bg-slate-900 bg-opacity-75 transition-opacity" />
			</TransitionChild>

			<div class="fixed inset-0 z-10 w-screen overflow-y-auto">
				<div class="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
					<TransitionChild
						as="template"
						enter="ease-out duration-300"
						enter-from="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
						enter-to="opacity-100 translate-y-0 sm:scale-100"
						leave="ease-in duration-200"
						leave-from="opacity-100 translate-y-0 sm:scale-100"
						leave-to="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
					>
						<DialogPanel
							class="relative transform overflow-hidden rounded-lg bg-slate-900 px-4 pb-4 pt-5 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-sm sm:p-6"
						>
							<div>
								<div
									class="mx-auto flex h-12 w-12 items-center justify-center rounded-full"
								>
									<QuestionMarkCircleIcon class="h-6 w-6 text-slate-200" />
								</div>
								<div class="mt-3 text-center sm:mt-5">
									<DialogTitle as="h3" class="text-base font-semibold leading-6 text-slate-200">What side do you want to play on?</DialogTitle>
								</div>
							</div>
							<div class="mt-5 sm:mt-6 flex space-x-2">
								<AppButton @click="$emit('chosen', Piece.Max)" class="!bg-blue-500">
									Blue
								</AppButton>
								<AppButton @click="$emit('chosen', Piece.Min)" class="!bg-red-500">
									Red
								</AppButton>
							</div>
						</DialogPanel>
					</TransitionChild>
				</div>
			</div>
		</Dialog>
	</TransitionRoot>
</template>

<script setup lang="ts">
import { QuestionMarkCircleIcon } from '@heroicons/vue/24/outline';
import { Dialog, DialogPanel, DialogTitle, TransitionChild, TransitionRoot } from "@headlessui/vue";
import { Piece } from '@/stores/GameState';
import AppButton from './AppButton.vue';


const emit = defineEmits(['chosen']);
const props = defineProps<{
	open: boolean
}>()


</script>