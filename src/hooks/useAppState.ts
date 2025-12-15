import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { useEffect, useState } from "preact/hooks"
import type { AppState, Config, LanguageInfo, ModelInfo, Theme } from "../types"
import { getInitialTheme } from "../utils"

export function useAppState() {
	const [appState, setAppState] = useState<AppState>("idle")
	const [config, setConfig] = useState<Config | null>(null)
	const [models, setModels] = useState<ModelInfo[]>([])
	const [hasModel, setHasModel] = useState(false)
	const [isMultilingual, setIsMultilingual] = useState(false)
	const [lastTranscription, setLastTranscription] = useState("")
	const [error, setError] = useState<string | null>(null)
	const [modelsDir, setModelsDir] = useState("")
	const [theme, setTheme] = useState<Theme>(getInitialTheme)
	const [supportedLanguages, setSupportedLanguages] = useState<LanguageInfo[]>(
		[]
	)

	useEffect(() => {
		loadSupportedLanguages()
		loadInitialData()
		setupEventListeners()
	}, [])

	useEffect(() => {
		localStorage.setItem("theme", theme)
		document.documentElement.setAttribute("data-theme", theme)
	}, [theme])

	async function loadSupportedLanguages() {
		try {
			const languages = await invoke<LanguageInfo[]>("get_supported_languages")
			setSupportedLanguages(languages)
		} catch (e) {
			setError(String(e))
		}
	}

	async function loadInitialData() {
		try {
			const [configData, modelsData, hasModelData, modelsDirectory] =
				await Promise.all([
					invoke<Config>("get_config"),
					invoke<ModelInfo[]>("get_available_models"),
					invoke<boolean>("has_model_loaded"),
					invoke<string>("get_models_directory")
				])
			setConfig(configData)
			setModels(modelsData)
			setHasModel(hasModelData)
			setModelsDir(modelsDirectory)

			if (hasModelData) {
				const multilingual = await invoke<boolean>("is_model_multilingual")
				setIsMultilingual(multilingual)
			}
		} catch (e) {
			setError(String(e))
		}
	}

	function setupEventListeners() {
		listen<AppState>("state-changed", (event) => {
			setAppState(event.payload)
		})

		listen<string>("transcription", (event) => {
			setLastTranscription(event.payload)
		})

		listen<string>("error", (event) => {
			setError(event.payload)
		})
	}

	return {
		appState,
		config,
		setConfig,
		models,
		hasModel,
		setHasModel,
		isMultilingual,
		setIsMultilingual,
		lastTranscription,
		error,
		setError,
		modelsDir,
		theme,
		setTheme,
		supportedLanguages,
		loadInitialData
	}
}
