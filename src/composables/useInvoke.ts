import { invoke } from '@tauri-apps/api/core';

interface InvokeResult {
	output: any
	time: number
}

export async function useInvoke(func_name: string, ...args: any[]): Promise<InvokeResult> {
	const start = performance.now()
	try {
		const result = await invoke(func_name, ...args)
		return {
			output: result,
			time: performance.now() - start,
		}
	} catch (e) {
		console.error(`invoke ${func_name} failed: ${e}`)
		throw e
	}
}
