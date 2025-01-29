<template>
	<div class="wrapper">
		<span>Settings Page</span>
		<span>Config Dir: {{ settingsDir }}</span>
		{{ userSettings }}
	</div>
</template>

<script setup lang="ts">
import { useAppSettingsStore } from '../store/AppSettings'

const { userSettings, settingsDir } = useAppSettingsStore()
</script>

<style scoped>
@reference "tailwindcss";

.wrapper {
	@apply px-10;
}
</style>
