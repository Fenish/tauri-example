<template>
	<div class="flex flex-col items-center justify-center min-h-screen p-4">
		<div class="bg-zinc-800 p-6 rounded-lg shadow-lg max-w-md w-full">
			<h2 class="text-2xl font-bold mb-6 text-center">C++ Sum Calculator</h2>

			<div class="space-y-4">
				<div>
					<label class="block text-sm font-medium mb-1">First Number</label>
					<input
						v-model="num1"
						type="number"
						class="w-full px-3 py-2 bg-zinc-700 rounded border border-zinc-600 focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
						placeholder="Enter first number"
					/>
				</div>

				<div>
					<label class="block text-sm font-medium mb-1">Second Number</label>
					<input
						v-model="num2"
						type="number"
						class="w-full px-3 py-2 bg-zinc-700 rounded border border-zinc-600 focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
						placeholder="Enter second number"
					/>
				</div>

				<button
					@click="calculateSum"
					class="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded transition-colors"
				>
					Calculate Sum
				</button>

				<div v-if="result !== null" class="mt-4 text-center">
					<p class="text-xl">
						Result:
						<span class="font-bold">{{ result }}</span>
					</p>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const num1 = ref<number>(0)
const num2 = ref<number>(0)
const result = ref<number | null>(null)

const calculateSum = async () => {
	try {
		result.value = await invoke('calculate_sum', {
			a: parseInt(num1.value.toString()),
			b: parseInt(num2.value.toString()),
		})
	} catch (error) {
		console.error('Error calculating sum:', error)
	}
}
</script>
