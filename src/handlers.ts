import { invoke } from "@tauri-apps/api/core"
import { open } from "@tauri-apps/plugin-dialog"
import { openUrl } from "@tauri-apps/plugin-opener"
import type { Config } from "./types"

export async function handleLoadModel(
	setHasModel: (value: boolean) => void,
	setError: (value: string | null) => void,
	setIsMultilingual: (value: boolean) => void,
	loadInitialData: () => Promise<void>
) {
	try {
		const selected = await open({
			multiple: false,
			filters: [{ name: "Whisper Model", extensions: ["bin"] }],
			directory: false
		})

		if (selected) {
			await invoke("load_model", { modelPath: selected })
			setHasModel(true)
			setError(null)
			const multilingual = await invoke<boolean>("is_model_multilingual")
			setIsMultilingual(multilingual)
			await loadInitialData()
		}
	} catch (e) {
		setError(String(e))
	}
}

export async function handleSelectModel(
	modelPath: string,
	setHasModel: (value: boolean) => void,
	setError: (value: string | null) => void,
	setIsMultilingual: (value: boolean) => void,
	loadInitialData: () => Promise<void>
) {
	if (!modelPath) return
	try {
		await invoke("load_model", { modelPath })
		setHasModel(true)
		setError(null)
		const multilingual = await invoke<boolean>("is_model_multilingual")
		setIsMultilingual(multilingual)
		await loadInitialData()
	} catch (e) {
		setError(String(e))
	}
}

export async function saveConfig(
	config: Config,
	setError: (value: string | null) => void
) {
	try {
		await invoke("save_config", { config })
		setError(null)
	} catch (e) {
		setError(String(e))
	}
}

export function updateConfig<K extends keyof Config>(
	config: Config | null,
	setConfig: (value: Config | null) => void,
	key: K,
	value: Config[K]
): Config | null {
	if (config) {
		const newConfig = { ...config, [key]: value }
		setConfig(newConfig)
		return newConfig
	}
	return null
}

export async function updateAndSaveConfig<K extends keyof Config>(
	config: Config | null,
	setConfig: (value: Config | null) => void,
	setError: (value: string | null) => void,
	key: K,
	value: Config[K]
) {
	const newConfig = updateConfig(config, setConfig, key, value)
	if (newConfig) {
		await saveConfig(newConfig, setError)
	}
}

export async function openModelUrl() {
	try {
		await openUrl("https://huggingface.co/ggerganov/whisper.cpp/tree/main")
	} catch (e) {
		console.error("Failed to open URL:", e)
	}
}
