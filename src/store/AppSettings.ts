import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

import { appConfigDir } from '@tauri-apps/api/path'
import { exists, BaseDirectory, create, mkdir, readTextFile } from '@tauri-apps/plugin-fs'

import defaultUserSettings from '../../settings/userSettings.json'
import defaultGlobalSettings from '../../settings/globalSettings.json'

class SettingsFile {
	private filename: string
	private settings: any

	constructor(filename: string) {
		this.filename = filename
	}

	async createIfNotExists(defaultContent: any = {}) {
		const isExists = await exists(this.filename, { baseDir: BaseDirectory.AppConfig })
		let file
		if (!isExists) {
			file = await create(this.filename, { baseDir: BaseDirectory.AppConfig })
			await file.write(new TextEncoder().encode(defaultContent))
			await file.close()
		} else {
			this.settings = await this.getContent()
		}
	}

	async getContent() {
		const content = await readTextFile(this.filename, { baseDir: BaseDirectory.AppConfig })
		return JSON.parse(content)
	}

	get(key: string): any {
		return this.settings[key]
	}

	set(key: string, value: any): void {
		this.settings[key] = value
	}

	get keys(): string[] {
		return Object.keys(this.settings)
	}

	get values(): any[] {
		return Object.values(this.settings)
	}

	toString(): string {
		return JSON.stringify(this.settings, null, 2)
	}
}

export const useAppSettingsStore = defineStore('AppSettings', () => {
	const settingsDir = ref('')
	const userSettings = ref<SettingsFile | null>(null) // Make it a ref
	const globalSettings = ref<SettingsFile | null>(null)

	async function init() {
		settingsDir.value = await appConfigDir()
		if (!(await exists(settingsDir.value))) {
			await mkdir('', { baseDir: BaseDirectory.AppConfig })
		}
		await checkSettingsFiles()
	}

	async function checkSettingsFiles() {
		// Initialize userSettings after the file is created
		userSettings.value = new SettingsFile('user_settings.json')
		globalSettings.value = new SettingsFile('global_settings.json')

		await userSettings.value.createIfNotExists(defaultUserSettings)
		await globalSettings.value.createIfNotExists(defaultGlobalSettings)
	}

	return {
		init,
		settingsDir,
		userSettings: computed(() => userSettings.value),
		globalSettings: computed(() => globalSettings),
	}
})
