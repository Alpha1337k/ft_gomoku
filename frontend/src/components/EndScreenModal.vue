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
									:class="player == Piece.Max ? 'bg-blue-600' : 'bg-red-600'"
								>
									<TrophyIcon class="h-6 w-6 text-slate-200" />
								</div>
								<div class="mt-3 text-center sm:mt-5">
									<DialogTitle as="h3" class="text-base font-semibold leading-6 text-slate-200">{{ playerName }} has won!</DialogTitle>
									<div class="mt-2">
										<p class="text-sm text-slate-500">Totally not an landslide.</p>
									</div>
								</div>
							</div>
							<div class="mt-5 sm:mt-6">
								<AppButton @click="$emit('newGame')">
									Try again?
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
import { ref } from "vue";
import { Dialog, DialogPanel, DialogTitle, TransitionChild, TransitionRoot } from "@headlessui/vue";
import { TrophyIcon } from "@heroicons/vue/24/outline";
import { Piece } from "@/stores/GameState";
import AppButton from '@/components/AppButton.vue';

const props = defineProps<{
	open: boolean;
	player: Piece;
	playerName: string;
}>();

const emit = defineEmits(["newGame"]);
</script>
