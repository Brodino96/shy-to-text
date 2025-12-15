interface TranscriptionSectionProps {
	lastTranscription: string
}

export function TranscriptionSection({
	lastTranscription
}: TranscriptionSectionProps) {
	return (
		<div class="section">
			<h2>Last Transcription</h2>
			<div class={`transcription-box ${!lastTranscription ? "empty" : ""}`}>
				{lastTranscription ||
					"No transcription yet. Press the hotkey to start recording."}
			</div>
		</div>
	)
}
