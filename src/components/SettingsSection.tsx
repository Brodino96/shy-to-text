import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { sendNotification } from "@tauri-apps/plugin-notification"
import { useCallback, useEffect, useState } from "preact/hooks"
import type { Config, GpuDevice, LanguageInfo } from "../types"

interface SettingsSectionProps {
	config: Config
	hasModel: boolean
	isMultilingual: boolean
	supportedLanguages: LanguageInfo[]
	saveConfig: (config: Config) => Promise<void>
}

const VALID_KEYS = new Set([
	"F1",
	"F2",
	"F3",
	"F4",
	"F5",
	"F6",
	"F7",
	"F8",
	"F9",
	"F10",
	"F11",
	"F12",
	"A",
	"B",
	"C",
	"D",
	"E",
	"F",
	"G",
	"H",
	"I",
	"J",
	"K",
	"L",
	"M",
	"N",
	"O",
	"P",
	"Q",
	"R",
	"S",
	"T",
	"U",
	"V",
	"W",
	"X",
	"Y",
	"Z",
	"0",
	"1",
	"2",
	"3",
	"4",
	"5",
	"6",
	"7",
	"8",
	"9",
	"Space"
])

function keyEventToHotkey(e: KeyboardEvent): string | null {
	const modifiers: string[] = []

	if (e.ctrlKey) modifiers.push("Ctrl")
	if (e.altKey) modifiers.push("Alt")
	if (e.shiftKey) modifiers.push("Shift")
	if (e.metaKey) modifiers.push("Super")

	let key: string | null = null
	let isFunctionKey = false

	if (e.code.startsWith("Key")) {
		key = e.code.slice(3)
	} else if (e.code.startsWith("Digit")) {
		key = e.code.slice(5)
	} else if (e.code.startsWith("F") && /^F\d+$/.test(e.code)) {
		key = e.code
		isFunctionKey = true
	} else if (e.code === "Space") {
		key = "Space"
	}

	if (key && VALID_KEYS.has(key)) {
		// Require a modifier for non-function keys
		if (!isFunctionKey && modifiers.length === 0) {
			return null
		}
		return [...modifiers, key].join("+")
	}

	return null
}

export function SettingsSection({
	config,
	hasModel,
	isMultilingual,
	supportedLanguages,
	saveConfig
}: SettingsSectionProps) {
	const [pendingConfig, setPendingConfig] = useState<Config>(config)
	const [isRecording, setIsRecording] = useState(false)
	const [gpuDevices, setGpuDevices] = useState<GpuDevice[]>([])

	useEffect(() => {
		setPendingConfig(config)
	}, [config])

	// Fetch available GPU devices on mount
	useEffect(() => {
		invoke<GpuDevice[]>("get_gpu_devices").then(setGpuDevices)
	}, [])

	// Listen for GPU fallback event and refetch config
	useEffect(() => {
		const unlisten = listen("gpu-fallback", async () => {
			const newConfig = await invoke<Config>("get_config")
			setPendingConfig(newConfig)
		})
		return () => {
			unlisten.then((fn) => fn())
		}
	}, [])

	const handleKeyDown = useCallback((e: KeyboardEvent) => {
		e.preventDefault()
		e.stopPropagation()

		if (e.key === "Escape") {
			setIsRecording(false)
			return
		}

		const hotkey = keyEventToHotkey(e)
		if (hotkey) {
			setPendingConfig((prev) => ({ ...prev, hotkey }))
			setIsRecording(false)
		}
	}, [])

	useEffect(() => {
		if (isRecording) {
			window.addEventListener("keydown", handleKeyDown)
			return () => window.removeEventListener("keydown", handleKeyDown)
		}
	}, [isRecording, handleKeyDown])

	const availableLanguages = isMultilingual
		? [{ code: "auto", name: "Auto-detect" }, ...supportedLanguages]
		: supportedLanguages.filter((lang) => lang.code === "en")

	const hasChanges =
		pendingConfig.hotkey !== config.hotkey ||
		pendingConfig.language !== config.language ||
		pendingConfig.auto_copy !== config.auto_copy ||
		pendingConfig.show_notifications !== config.show_notifications ||
		pendingConfig.use_gpu !== config.use_gpu ||
		pendingConfig.gpu_device !== config.gpu_device

	function getLanguageName(code: string): string {
		if (code === "auto") return "Auto-detect"
		const lang = supportedLanguages.find((l) => l.code === code)
		return lang?.name ?? code
	}

	function getGpuDeviceLabel(id: number): string {
		const device = gpuDevices.find((d) => d.id === id)
		return device ? `${device.name} (${device.backend})` : ""
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
		if (pendingConfig.use_gpu !== config.use_gpu) {
			changes.push(
				`GPU: ${config.use_gpu ? "On" : "Off"} -> ${pendingConfig.use_gpu ? "On" : "Off"}`
			)
		}
		if (pendingConfig.gpu_device !== config.gpu_device) {
			const oldDevice = gpuDevices.find((d) => d.id === config.gpu_device)
			const newDevice = gpuDevices.find(
				(d) => d.id === pendingConfig.gpu_device
			)
			changes.push(
				`GPU Device: ${oldDevice?.name ?? config.gpu_device} -> ${newDevice?.name ?? pendingConfig.gpu_device}`
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
					<span class="setting-label">Hotkey</span>
					<div class="hotkey-recorder">
						<span class={`hotkey-display ${isRecording ? "recording" : ""}`}>
							{isRecording ? "Press a key combo..." : pendingConfig.hotkey}
						</span>
						<button
							type="button"
							class={isRecording ? "recording" : ""}
							onClick={() => setIsRecording(!isRecording)}
						>
							{isRecording ? "Cancel" : "Record"}
						</button>
					</div>
				</div>

				<div class="setting-row">
					<label for="language-select">Language</label>
					<select
						id="language-select"
						value={pendingConfig.language}
						title={getLanguageName(pendingConfig.language)}
						onChange={(e) =>
							setPendingConfig({
								...pendingConfig,
								language: e.currentTarget.value
							})
						}
						disabled={!hasModel}
					>
						{availableLanguages.map((lang) => (
							<option key={lang.code} value={lang.code} title={lang.name}>
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

				<div class="setting-row checkbox-row">
					<input
						type="checkbox"
						id="use-gpu"
						checked={pendingConfig.use_gpu}
						disabled={gpuDevices.length === 0}
						onChange={(e) =>
							setPendingConfig({
								...pendingConfig,
								use_gpu: e.currentTarget.checked
							})
						}
					/>
					<label for="use-gpu">Use GPU acceleration</label>
					{gpuDevices.length === 0 && (
						<span class="setting-hint" style={{ color: "red" }}>
							No GPU devices available
						</span>
					)}
				</div>

				<div class="setting-row">
					<label for="gpu-device-select">GPU Device</label>
					<select
						id="gpu-device-select"
						value={pendingConfig.gpu_device}
						title={getGpuDeviceLabel(pendingConfig.gpu_device)}
						disabled={!pendingConfig.use_gpu || gpuDevices.length === 0}
						onChange={(e) =>
							setPendingConfig({
								...pendingConfig,
								gpu_device: Number(e.currentTarget.value)
							})
						}
					>
						{gpuDevices.map((device) => {
							const label = `${device.name} (${device.backend})`
							return (
								<option key={device.id} value={device.id} title={label}>
									{label}
								</option>
							)
						})}
					</select>
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
