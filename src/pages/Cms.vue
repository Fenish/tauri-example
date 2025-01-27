<template>
	<div class="cms-container">
		<!-- RGB Input Section -->
		<div class="input-section">
			<div class="rgb-inputs">
				<div class="input-group">
					<label>R</label>
					<input type="number" min="0" max="255" placeholder="0-255" v-model="rgb.r" />
				</div>
				<div class="input-group">
					<label>G</label>
					<input type="number" min="0" max="255" placeholder="0-255" v-model="rgb.g" />
				</div>
				<div class="input-group">
					<label>B</label>
					<input type="number" min="0" max="255" placeholder="0-255" v-model="rgb.b" />
				</div>
			</div>

			<!-- Conversion Options -->
			<div class="conversion-options">
				<div class="radio-group">
					<input type="radio" id="cmyk" name="conversion" value="cmyk" v-model="selectedMode" />
					<label for="cmyk">CMYK</label>
				</div>

				<div class="radio-group">
					<input type="radio" id="lab" name="conversion" value="lab" v-model="selectedMode" />
					<label for="lab">LAB</label>
				</div>
			</div>

			<!-- Dynamic Input Section -->
			<div class="dynamic-inputs">
				<!-- CMYK Inputs -->
				<div v-if="selectedMode === 'cmyk'" class="input-set cmyk-inputs">
					<div v-for="(value, index) in cmykValues" :key="index" class="input-group">
						<label>{{ ['C', 'M', 'Y', 'K'][index] }}</label>
						<input type="number" min="0" max="100" placeholder="0-100" v-model="cmykValues[index]" />
					</div>
				</div>

				<!-- LAB Inputs -->
				<div v-if="selectedMode === 'lab'" class="input-set lab-inputs">
					<div v-for="(value, index) in labValues" :key="index" class="input-group">
						<label>{{ ['L', 'A', 'B'][index] }}</label>
						<input type="number" min="0" max="100" placeholder="0-100" v-model="labValues[index]" />
					</div>
				</div>
			</div>

			<button class="convert-btn" @click="convertColor">Convert</button>
		</div>

		<!-- Color Preview Section -->
		<div class="preview-section">
			<div class="color-box">
				<h3>Source RGB</h3>
				<div class="color-preview source-preview" :style="{ backgroundColor: sourceColor }"></div>
				<div class="color-code">{{ sourceColorCode }}</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useInvoke } from '../composables/useInvoke'

const selectedMode = ref('cmyk')
const rgb = ref({ r: 0, g: 0, b: 0 })
const cmykValues = ref([0, 0, 0, 0]) // Array for CMYK inputs
const labValues = ref([0, 0, 0]) // Array for LAB inputs

const sourceColor = computed(() => {
	return `rgb(${rgb.value.r}, ${rgb.value.g}, ${rgb.value.b})`
})

const sourceColorCode = computed(() => {
	return `RGB(${rgb.value.r}, ${rgb.value.g}, ${rgb.value.b})`
})

async function convertColor() {
	let command = 'rgb_to_cmyk'
	if (selectedMode.value === 'lab') {
		command = 'rgb_to_lab'
	}

	try {
		const { output, time } = await useInvoke(command, {
			r: rgb.value.r,
			g: rgb.value.g,
			b: rgb.value.b,
		})

		if (selectedMode.value === 'cmyk') {
			cmykValues.value = output
		} else {
			labValues.value = output
		}
	} catch (error) {
		console.error(error)
	}
}
</script>

<style scoped>
@reference "tailwindcss";

.cms-container {
	@apply w-full h-full items-center justify-center p-8 flex flex-col gap-8;
}

.input-section {
	@apply w-full max-w-3xl mx-auto bg-zinc-800 p-6 rounded-lg shadow-md;
}

.rgb-inputs {
	@apply flex gap-4 mb-8;
}

.input-group {
	@apply flex flex-col gap-1;
}

.input-group label {
	@apply text-sm font-medium text-white;
}

.input-group input {
	@apply w-24 px-3 py-2 border border-zinc-700 rounded-md bg-zinc-900 text-white focus:ring-2 focus:ring-purple-500 focus:border-purple-500;
}

.conversion-options {
	@apply flex gap-8 mb-8;
}

.radio-group {
	@apply flex items-center gap-2;
}

.radio-group input[type='radio'] {
	@apply w-7 h-7 text-purple-600 bg-zinc-900 border-zinc-700;
}

.radio-group label {
	@apply text-sm font-medium text-white;
}

.dynamic-inputs {
	@apply mb-6;
}

.input-set {
	@apply flex flex-wrap gap-4 flex-row;
}

.convert-btn {
	@apply w-full py-2 px-4 bg-purple-600 text-white font-medium rounded-md hover:bg-purple-700 transition-colors;
}

.preview-section {
	@apply w-full max-w-3xl mx-auto flex gap-8 justify-center;
}

.color-box {
	@apply flex-1 bg-zinc-800 p-6 rounded-lg shadow-md;
}

.color-box h3 {
	@apply text-lg font-medium text-white mb-4;
}

.color-preview {
	@apply w-full h-32 rounded-md border border-zinc-700 mb-4;
}

.source-preview {
	@apply bg-black;
}

.target-preview {
	@apply bg-black;
}

.color-code {
	@apply text-sm font-mono text-white text-center;
}
</style>
