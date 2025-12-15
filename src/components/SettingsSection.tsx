import { sendNotification } from "@tauri-apps/plugin-notification"
import { useEffect, useState } from "preact/hooks"
import type { Config, LanguageInfo } from "../types"

interface SettingsSectionProps {
	config: Config
	hasModel: boolean
	isMultilingual: boolean
	supportedLanguages: LanguageInfo[]
	saveConfig: (config: Config) => Promise<void>
}

export function SettingsSection({
	config,
	hasModel,
	isMultilingual,
	supportedLanguages,
	saveConfig
}: SettingsSectionProps) {
	const [pendingConfig, setPendingConfig] = useState<Config>(config)

	useEffect(() => {
		setPendingConfig(config)
	}, [config])

	const availableLanguages = isMultilingual
		? [{ code: "auto", name: "Auto-detect" }, ...supportedLanguages]
		: supportedLanguages.filter((lang) => lang.code === "en")

	const hasChanges =
		pendingConfig.hotkey !== config.hotkey ||
		pendingConfig.language !== config.language ||
		pendingConfig.auto_copy !== config.auto_copy ||
		pendingConfig.show_notifications !== config.show_notifications

	function getLanguageName(code: string): string {
		if (code === "auto") return "Auto-detect"
		const lang = supportedLanguages.find((l) => l.code === code)
		return lang?.name ?? code
	}

	async function handleApply() {
		const changes: string[] = []

		if (pendingConfig.hotkey !== config.hotkey) {
			changes.push(`Hotkey: "${config.hotkey}" -> "${pendingConfig.hotkey}"`)
		}
		if (pendingConfig.language !== config.language) {
			changes.push(
				`Language: ${getLanguageName(config.language)} -> ${getLanguageName(pendingConfig.language)}`
			)
		}
		if (pendingConfig.auto_copy !== config.auto_copy) {
			changes.push(
				`Auto-copy: ${config.auto_copy ? "On" : "Off"} -> ${pendingConfig.auto_copy ? "On" : "Off"}`
			)
		}
		if (pendingConfig.show_notifications !== config.show_notifications) {
			changes.push(
				`Notifications: ${config.show_notifications ? "On" : "Off"} -> ${pendingConfig.show_notifications ? "On" : "Off"}`
			)
		}

		await saveConfig(pendingConfig)

		if (changes.length > 0) {
			try {
				sendNotification({
					title: "Settings Updated",
					body: changes.join("\n")
				})
			} catch (e) {
				console.error("Failed to send notification:", e)
			}
		}
	}

	return (
		<div class="section">
			<h2>Settings</h2>
			<div class="settings-grid">
				<div class="setting-row">
					<label for="hotkey-input">Hotkey</label>
					<input
						id="hotkey-input"
						type="text"
						value={pendingConfig.hotkey}
						onInput={(e) =>
							setPendingConfig({
								...pendingConfig,
								hotkey: e.currentTarget.value
							})
						}
						placeholder="e.g., F9 or Ctrl+Shift+R"
					/>
				</div>

				<div class="setting-row">
					<label for="language-select">Language</label>
					<select
						id="language-select"
						value={pendingConfig.language}
						onChange={(e) =>
							setPendingConfig({
								...pendingConfig,
								language: e.currentTarget.value
							})
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
						checked={pendingConfig.auto_copy}
						onChange={(e) =>
							setPendingConfig({
								...pendingConfig,
								auto_copy: e.currentTarget.checked
							})
						}
					/>
					<label for="auto-copy">Auto-copy transcription to clipboard</label>
				</div>

				<div class="setting-row checkbox-row">
					<input
						type="checkbox"
						id="show-notifications"
						checked={pendingConfig.show_notifications}
						onChange={(e) =>
							setPendingConfig({
								...pendingConfig,
								show_notifications: e.currentTarget.checked
							})
						}
					/>
					<label for="show-notifications">Show notifications</label>
				</div>

				<div class="setting-row">
					<button
						type="button"
						class="primary apply-button"
						onClick={handleApply}
						disabled={!hasChanges}
					>
						Apply
					</button>
				</div>
			</div>
		</div>
	)
}
