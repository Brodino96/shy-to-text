import "./App.css"
import { Header } from "./components/Header"
import { HotkeyHint } from "./components/HotkeyHint"
import { ModelSection } from "./components/ModelSection"
import { SettingsSection } from "./components/SettingsSection"
import { StatusIndicator } from "./components/StatusIndicator"
import { ThemeSwitch } from "./components/ThemeSwitch"
import { TranscriptionSection } from "./components/TranscriptionSection"
import {
	handleLoadModel,
	handleSelectModel,
	openModelUrl,
	saveConfig,
	updateAndSaveConfig,
	updateConfig
} from "./handlers"
import { useAppState } from "./hooks/useAppState"
import type { Config } from "./types"

function App() {
	const {
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
	} = useAppState()

	const handleLoadModelWrapper = () =>
		handleLoadModel(setHasModel, setError, setIsMultilingual, loadInitialData)
	const handleSelectModelWrapper = (modelPath: string) =>
		handleSelectModel(
			modelPath,
			setHasModel,
			setError,
			setIsMultilingual,
			loadInitialData
		)
	const openModelUrlWrapper = () => openModelUrl()
	const saveConfigWrapper = (newConfig: Config) =>
		saveConfig(newConfig, setError)
	const updateConfigWrapper = <K extends keyof Config>(
		key: K,
		value: Config[K]
	) => updateConfig(config, setConfig, key, value)
	const updateAndSaveConfigWrapper = <K extends keyof Config>(
		key: K,
		value: Config[K]
	) => updateAndSaveConfig(config, setConfig, setError, key, value)

	return (
		<div class="app">
			<ThemeSwitch theme={theme} setTheme={setTheme} />

			<Header />

			<StatusIndicator appState={appState} />

			{error && <div class="error-message">{error}</div>}

			<TranscriptionSection lastTranscription={lastTranscription} />

			<ModelSection
				hasModel={hasModel}
				config={config}
				models={models}
				modelsDir={modelsDir}
				isMultilingual={isMultilingual}
				handleLoadModel={handleLoadModelWrapper}
				handleSelectModel={handleSelectModelWrapper}
				openModelUrl={openModelUrlWrapper}
			/>

			{config && (
				<SettingsSection
					config={config}
					hasModel={hasModel}
					isMultilingual={isMultilingual}
					supportedLanguages={supportedLanguages}
					updateConfig={updateConfigWrapper}
					updateAndSaveConfig={updateAndSaveConfigWrapper}
					saveConfig={saveConfigWrapper}
				/>
			)}

			<HotkeyHint config={config} />
		</div>
	)
}

export default App
