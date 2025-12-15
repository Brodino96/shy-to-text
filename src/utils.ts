import type { AppState, Theme } from "./types"

export function getInitialTheme(): Theme {
	const saved = localStorage.getItem("theme") as Theme | null
	if (saved) return saved
	return window.matchMedia("(prefers-color-scheme: dark)").matches
		? "dark"
		: "light"
}

export function getStatusText(appState: AppState): string {
	switch (appState) {
		case "idle":
			return "Ready"
		case "recording":
			return "Recording..."
		case "transcribing":
			return "Transcribing..."
	}
}

export function formatFileSize(bytes: number): string {
	if (bytes < 1024 * 1024) {
		return `${(bytes / 1024).toFixed(1)} KB`
	}
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
