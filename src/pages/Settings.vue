<template>
	<div class="wrapper">
		<span>Settings Page</span>
		<span>Config Dir: {{ configRef }}</span>
	</div>
</template>

<script setup lang="ts">
import { appConfigDir } from '@tauri-apps/api/path'
import { ref, onMounted } from 'vue'

const configRef = ref<any>(null)

onMounted(async () => {
	configRef.value = await appConfigDir()
})
</script>

<style scoped>
@reference "tailwindcss";

.wrapper {
	@apply px-10;
}
</style>
