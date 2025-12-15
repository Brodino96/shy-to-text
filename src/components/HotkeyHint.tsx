import type { Config } from "../types"

interface HotkeyHintProps {
	config: Config | null
}

export function HotkeyHint({ config }: HotkeyHintProps) {
	return (
		<div class="hotkey-hint">
			Press <kbd>{config?.hotkey || "F9"}</kbd> to start/stop recording
		</div>
	)
}
