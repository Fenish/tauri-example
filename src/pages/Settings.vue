<template>
	<div class="wrapper">
		<span>{{ settingsDir }}</span>
		<div class="image-area">
			<button class="import-button" @click="test">Import Image</button>
			<div class="w-full rounded-md bg-zinc-700 h-2">
				<div class="bg-green-500 h-full transition-all duration-300" :style="{ width: imageProgress + '%' }" />
			</div>
			<span>Time took</span>
			<span>{{ timetook }}</span>
			<div class="w-full border h-full p-3">
				<div v-for="image in images" :key="image">
					<span>{{ image }}</span>
					<img :src="image.base64_data" class="w-32 h-32" />
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useAppSettingsStore } from '../store/AppSettings'
import { invoke, convertFileSrc, Channel } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref } from 'vue'

const { settingsDir } = useAppSettingsStore()

const imageProgress = ref<any>(0)
const images = ref<any>({})
const timetook = ref<any>('')

listen('image-loading-progress', (event) => {
	imageProgress.value = event.payload
	if (event.payload == 100) {
		setTimeout(() => {
			imageProgress.value = 0
		}, 1000)
	}
})

type DownloadEvent =
	| {
			event: 'started'
			data: {
				hash: string
				content_length: Array<number>
			}
	  }
	| {
			event: 'progress'
			data: {
				hash: string
				chunk: Array<number>
			}
	  }
	| {
			event: 'finished'
			data: {
				hash: string
			}
	  }

const channel = new Channel()
async function test() {
	const start = performance.now()
	const response = await invoke('load_and_resize_images', {
		channel: channel,
	})
	let diff: any = performance.now() - start
	// make diff minutes, seconds and miliseconds
	diff =
		Math.floor(diff / 1000 / 60) +
		' minutes ' +
		Math.floor((diff / 1000) % 60) +
		' seconds and ' +
		Math.floor(diff % 1000) +
		' miliseconds'
	timetook.value = diff
}

channel.onmessage = (event) => {
	console.log(event)
}
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
