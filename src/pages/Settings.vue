<template>
	<div class="wrapper">
		<span>Settings Page</span>
		<span>Config Dir: {{ settingsDir }}</span>
		<button @click="test">Test</button>
	</div>
</template>

<script setup lang="ts">
import { useAppSettingsStore } from '../store/AppSettings'
import { invoke } from '@tauri-apps/api/core'

const { settingsDir } = useAppSettingsStore()
async function test() {
	const response = await invoke('load_and_resize_images')
	console.log(response)
}
</script>

<style scoped>
@reference "tailwindcss";

.wrapper {
	@apply px-10;
}
</style>
