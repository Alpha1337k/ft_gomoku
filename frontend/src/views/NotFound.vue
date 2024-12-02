<script setup lang="ts">
import AppButton from '@/components/AppButton.vue';
import { RouterLink } from 'vue-router';
</script>

<template>
	<div class=" max-w-96 my-8 space-y-4">
		<h1 class="text-2xl">404 Not found</h1>
		<RouterLink to="/">
			<AppButton class="w-48">Back to the app</AppButton>
		</RouterLink>
	</div>

</template>