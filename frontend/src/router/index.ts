import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
	history: createWebHistory(import.meta.env.BASE_URL),
	routes: [
		{
			path: "/",
			name: "home",
			component: () => import("@/views/HomePage.vue"),
		},
		{
			path: "/bot",
			name: "Bot",
			component: () => import("@/views/AgainstBot.vue"),
		},
		{
			path: "/hotseat",
			name: "Hotseat",
			component: () => import("@/views/HotSeat.vue"),
		},
	],
});

export default router;
