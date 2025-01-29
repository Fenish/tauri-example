import { createRouter, createWebHistory } from 'vue-router'

const routes: Array<any> = [
	{
		path: '/',
		name: 'home',
		component: () => import('./layouts/Default.vue'),
		children: [
			{
				path: '',
				component: () => import('./pages/Home.vue'),
			},
			{
				path: 'settings',
				component: () => import('./pages/Settings.vue'),
			},
		],
	},
]

export const router = createRouter({
	history: createWebHistory(),
	routes,
})
