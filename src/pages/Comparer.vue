<template>
	<div class="wrapper">
		<span>{{ progressText }}</span>
		<div class="w-full h-5 bg-zinc-800 rounded-md">
			<div class="h-full bg-orange-500 transition-all duration-300" :style="{ width: progress + '%' }"></div>
		</div>
		<div class="image-area">
			<div class="flex flex-col gap-5">
				<button class="import-button rust-import-button" @click="importWithRust">
					Import Image With Rust
					<br />
					({{ timeCalcs.rust }})
				</button>

				<div class="border-2 border-green-900 p-3 rounded-md">
					<img :src="convertFileSrc(images.rust)" class="w-64 h-64" />
				</div>
			</div>

			<!-- <div class="flex flex-col gap-5">
				<button class="import-button py-import-button" @click="importWithPython">
					Import Image With Python
					<br />
					({{ timeCalcs.python }})
				</button>

				<div class="border-2 border-green-900 p-3 rounded-md">
					<img :src="convertFileSrc(images.python)" class="w-64 h-64" />
				</div>
			</div>

			<div class="flex flex-col gap-5">
				<button class="import-button cpp-import-button" @click="importWithRust">
					Import Image With C++
					<br />
					({{ timeCalcs.cpp }})
				</button>

				<div class="border-2 border-green-900 p-3 rounded-md">
					<img :src="convertFileSrc(images.cpp)" class="w-64 h-64" />
				</div>
			</div> -->
		</div>

		<div class="border-2 border-green-900 p-3 rounded-md">
			<h1 class="text-2xl font-bold">RUST Data</h1>
			<pre class="text-sm overflow-auto whitespace-pre-wrap bg-zinc-800 p-4 rounded-md mt-2">
				{{ data.rust }}
			</pre
			>
		</div>
	</div>
</template>

<script setup lang="ts">
import { invoke, Channel, convertFileSrc } from '@tauri-apps/api/core'
import { callFunction } from 'tauri-plugin-python-api'

import { onMounted, ref } from 'vue'

const progress = ref(0)
const progressText = ref('')

const images = ref<any>({
	rust: '',
	python: '',
	cpp: '',
})
const timeCalcs = ref<any>({
	rust: '',
	python: '',
	cpp: '',
})

const data = ref<any>({
	rust: '',
	python: '',
	cpp: '',
})

async function importWithRust() {
	const rustChannel = new Channel()
	rustChannel.onmessage = (event: any) => {
		if (event.event === 'complete') {
			timeCalcs.value['rust'] = event.data.time_taken
		} else if (event.event === 'progress') {
			progress.value = event.data.percentage
			progressText.value = event.data.step
			if (progress.value === 100) {
				setTimeout(() => {
					progress.value = 0
					progressText.value = ''
				}, 1000)
			}
		}
	}

	const response: any = await invoke('load_and_resize_images', {
		channel: rustChannel,
	})
	if (response.length === 0) return
	images.value['rust'] = response[0].paths.lowres
	data.value['rust'] = JSON.stringify(response, null, 2)
}

async function importWithPython() {
	const response = await callFunction('import_image_with_python', ['John'])
	console.log(response)
}

onMounted(() => {})
</script>

<style scoped>
@reference "tailwindcss";

.wrapper {
	@apply px-10;
}

.import-button {
	@apply p-2 border rounded-md text-white cursor-pointer min-w-24;
}

.rust-import-button {
	@apply bg-orange-500/50 border-orange-500 hover:bg-orange-500;
}

.py-import-button {
	@apply bg-yellow-500/50 border-yellow-500 hover:bg-yellow-500;
}

.cpp-import-button {
	@apply bg-blue-500/50 border-blue-500 hover:bg-blue-500;
}

.image-area {
	@apply flex gap-5 items-start;
}
</style>
