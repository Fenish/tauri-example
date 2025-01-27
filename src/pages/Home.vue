<template>
	<div class="flex flex-col items-center justify-center min-h-screen p-4">
		<div class="bg-zinc-800 p-6 rounded-lg shadow-lg max-w-md w-full">
			<h2 class="text-2xl font-bold mb-6 text-center">C++ Calculator</h2>

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

				<div class="grid grid-cols-2 gap-2">
					<button
						@click="calculate('sum')"
						class="bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded transition-colors"
					>
						Add (+)
					</button>
					<button
						@click="calculate('subtract')"
						class="bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded transition-colors"
					>
						Subtract (-)
					</button>
					<button
						@click="calculate('multiply')"
						class="bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded transition-colors"
					>
						Multiply (×)
					</button>
					<button
						@click="calculate('divide')"
						class="bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded transition-colors"
					>
						Divide (÷)
					</button>
				</div>

				<div class="border-t border-zinc-700 pt-4">
					<h3 class="text-lg font-semibold mb-2">Stress Test</h3>
					<div class="space-y-2">
						<div>
							<label class="block text-sm font-medium mb-1">Iterations</label>
							<input
								v-model="iterations"
								type="number"
								min="1000"
								step="1000"
								class="w-full px-3 py-2 bg-zinc-700 rounded border border-zinc-600 focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
								placeholder="Number of iterations (e.g., 100000)"
							/>
						</div>
						<button
							@click="runStressTest"
							:disabled="isRunning"
							class="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white font-medium py-2 px-4 rounded transition-colors"
						>
							<span v-if="isRunning">Running...</span>
							<span v-else>Run Stress Test</span>
						</button>
					</div>
				</div>

				<div v-if="result !== null" class="mt-4 text-center space-y-2">
					<p class="text-xl">
						Result:
						<span class="font-bold">{{ result }}</span>
					</p>
					<p class="text-sm text-gray-400">
						Elapsed time:
						<span class="font-mono">{{ elapsedTime }}ms</span>
					</p>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useInvoke } from '../composables/useInvoke'

const num1 = ref<number>(0)
const num2 = ref<number>(0)
const result = ref<number | null>(null)
const elapsedTime = ref<number>(0)
const iterations = ref<number>(100000)
const isRunning = ref<boolean>(false)

const calculate = async (operation: string) => {
	try {
		const a = parseInt(num1.value.toString())
		const b = parseInt(num2.value.toString())

		if (operation === 'divide' && b === 0) {
			alert('Cannot divide by zero!')
			return
		}

		const { output, time } = await useInvoke('calculate_' + operation, { a, b })
		console.log(`Result of ${operation}:`, output)
		elapsedTime.value = time
		result.value = output
	} catch (error) {
		console.error(`Error calculating ${operation}:`, error)
	}
}

const runStressTest = async () => {
	try {
		const { output, time } = await useInvoke('run_stress_test', { iterations: iterations.value })
		elapsedTime.value = time
		result.value = output
	} catch (error) {
		console.error('Error running stress test:', error)
	} finally {
		isRunning.value = false
	}
}
</script>
