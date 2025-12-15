import type { Config, LanguageInfo } from "../types"

interface SettingsSectionProps {
	config: Config
	hasModel: boolean
	isMultilingual: boolean
	supportedLanguages: LanguageInfo[]
	updateConfig: <K extends keyof Config>(
		key: K,
		value: Config[K]
	) => Config | null
	updateAndSaveConfig: <K extends keyof Config>(
		key: K,
		value: Config[K]
	) => Promise<void>
	saveConfig: (config: Config) => Promise<void>
}

export function SettingsSection({
	config,
	hasModel,
	isMultilingual,
	supportedLanguages,
	updateConfig,
	updateAndSaveConfig,
	saveConfig
}: SettingsSectionProps) {
	const availableLanguages = isMultilingual
		? [{ code: "auto", name: "Auto-detect" }, ...supportedLanguages]
		: supportedLanguages.filter((lang) => lang.code === "en")

	return (
		<div class="section">
			<h2>Settings</h2>
			<div class="settings-grid">
				<div class="setting-row">
					<label for="hotkey-input">Hotkey</label>
					<input
						id="hotkey-input"
						type="text"
						value={config.hotkey}
						onInput={(e) => updateConfig("hotkey", e.currentTarget.value)}
						onBlur={() => saveConfig(config)}
						placeholder="e.g., F9 or Ctrl+Shift+R"
					/>
				</div>

				<div class="setting-row">
					<label for="language-select">Language</label>
					<select
						id="language-select"
						value={config.language}
						onChange={(e) =>
							updateAndSaveConfig("language", e.currentTarget.value)
						}
						disabled={!hasModel}
					>
						{availableLanguages.map((lang) => (
							<option key={lang.code} value={lang.code}>
								{lang.name}
							</option>
						))}
					</select>
					{!isMultilingual && hasModel && (
						<span class="setting-hint">This model only supports English</span>
					)}
				</div>

				<div class="setting-row checkbox-row">
					<input
						type="checkbox"
						id="auto-copy"
						checked={config.auto_copy}
						onChange={(e) =>
							updateAndSaveConfig("auto_copy", e.currentTarget.checked)
						}
					/>
					<label for="auto-copy">Auto-copy transcription to clipboard</label>
				</div>

				<div class="setting-row checkbox-row">
					<input
						type="checkbox"
						id="show-notifications"
						checked={config.show_notifications}
						onChange={(e) =>
							updateAndSaveConfig("show_notifications", e.currentTarget.checked)
						}
					/>
					<label for="show-notifications">Show notifications</label>
				</div>
			</div>
		</div>
	)
}
