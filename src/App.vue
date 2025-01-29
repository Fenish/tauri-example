<template>
	<div class="bg-zinc-900 h-screen w-screen text-white overflow-hidden">
		<Titlebar />
		<RouterView />
	</div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppSettingsStore } from './store/AppSettings'

const { init } = useAppSettingsStore()
onMounted(async () => {
	await init()
})
</script>
