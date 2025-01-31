<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const currentRoute = computed(() => router.currentRoute.value)

const buttons = {
	Home: '/',
	Settings: '/Settings',
}

const isActiveRoute = (route: string) => {
	return currentRoute.value.path === route
}
</script>

<template>
	<div class="wrapper">
		<div class="menu">
			<button
				v-for="(value, key) in buttons"
				:key="key"
				class="menu-btn"
				:class="{ 'active-menu-btn': isActiveRoute(value) }"
				@click="router.push(value)"
			>
				{{ key }}
			</button>
		</div>
		<div>
			<RouterView />
		</div>
	</div>
</template>

<style scoped>
@reference "tailwindcss";

.wrapper {
	@apply w-full h-full py-10 flex flex-col gap-5 overflow-y-scroll;
}

.menu {
	@apply w-full flex gap-2 px-10;
}

.menu-btn {
	@apply p-2 bg-purple-500/50 border border-purple-500 hover:bg-purple-500 rounded-md text-white cursor-pointer min-w-24;
}

.active-menu-btn {
	@apply bg-purple-500;
}
</style>
