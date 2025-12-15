import type { AppState } from "../types"
import { getStatusText } from "../utils"

interface StatusIndicatorProps {
	appState: AppState
}

export function StatusIndicator({ appState }: StatusIndicatorProps) {
	return (
		<div class={`status-indicator ${appState}`}>
			<span class="status-dot" />
			<span>{getStatusText(appState)}</span>
		</div>
	)
}
