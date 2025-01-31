<template>
	<div class="wrapper">
		<span>{{ settingsDir }}</span>
		<div class="image-area">
			<button class="import-button" @click="test">Import Image</button>
			<div class="w-full rounded-md bg-zinc-700 h-2">
				<div class="bg-green-500 h-full transition-all duration-300" :style="{ width: imageProgress + '%' }" />
			</div>
			<div class="w-full border h-full p-3">
				<div v-for="image in images" :key="image">
					<span>{{ image }}</span>
					<img :src="convertFileSrc(image.lowres_path)" class="w-32 h-32" />
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useAppSettingsStore } from '../store/AppSettings'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref } from 'vue'

const { settingsDir } = useAppSettingsStore()

const imageProgress = ref<any>(0)
const images = ref<any>([])
async function test() {
	const response: Array<any> = await invoke('load_and_resize_images')
	images.value.push(...response)
}

listen('image-loading-progress', (event) => {
	imageProgress.value = event.payload
	if (event.payload == 100) {
		setTimeout(() => {
			imageProgress.value = 0
		}, 1000)
	}
})
</script>

<style scoped>
@reference "tailwindcss";

.wrapper {
	@apply px-10;
}

.import-button {
	@apply p-2 bg-green-500/50 border border-green-500 hover:bg-green-500 rounded-md text-white cursor-pointer min-w-24;
}

.image-area {
	@apply flex flex-col gap-5 items-start;
}
</style>
