export type AppState = "idle" | "recording" | "transcribing"

export interface Config {
	hotkey: string
	language: string
	model_path: string | null
	auto_copy: boolean
	show_notifications: boolean
	use_gpu: boolean
	gpu_device: number
}

export interface GpuDevice {
	id: number
	name: string
	device_type: string
	backend: string
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
