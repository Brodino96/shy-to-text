export type AppState = "idle" | "recording" | "transcribing"

export interface Config {
	hotkey: string
	language: string
	model_path: string | null
	auto_copy: boolean
	show_notifications: boolean
}

export interface ModelInfo {
	name: string
	path: string
	size: number
}

export interface LanguageInfo {
	code: string
	name: string
}

export type Theme = "light" | "dark"
