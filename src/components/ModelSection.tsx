import infoIcon from "../assets/info.svg"
import type { Config, ModelInfo } from "../types"
import { formatFileSize } from "../utils"

interface ModelSectionProps {
	hasModel: boolean
	config: Config | null
	models: ModelInfo[]
	modelsDir: string
	isMultilingual: boolean
	handleLoadModel: () => void
	handleSelectModel: (modelPath: string) => void
	openModelUrl: () => void
}

export function ModelSection({
	hasModel,
	config,
	models,
	modelsDir,
	isMultilingual,
	handleLoadModel,
	handleSelectModel,
	openModelUrl
}: ModelSectionProps) {
	return (
		<div class="section">
			<h2>
				Model
				<img
					src={infoIcon}
					alt="info"
					class="info-icon"
					title="The entire model will be loaded into RAM/VRAM. Choose a model size appropriate for your available memory."
				/>
			</h2>
			<div class="model-status">
				<span class={`model-name ${!hasModel ? "not-loaded" : ""}`}>
					{hasModel && config?.model_path
						? config.model_path.split("/").pop()
						: "No model loaded"}
				</span>
				<button type="button" class="secondary" onClick={handleLoadModel}>
					Browse...
				</button>
				<button type="button" class="secondary" onClick={openModelUrl}>
					Download models
				</button>
			</div>
			{hasModel && (
				<div class="model-info">
					{isMultilingual ? "Multilingual model" : "English-only model"}
				</div>
			)}
			{models.length > 0 && (
				<div style={{ marginTop: "12px" }}>
					<label
						for="model-select"
						style={{ fontSize: "0.85rem", color: "#666" }}
					>
						Available models in {modelsDir}:
					</label>
					<select
						id="model-select"
						style={{ width: "100%", marginTop: "6px" }}
						onChange={(e) => handleSelectModel(e.currentTarget.value)}
						value={config?.model_path || ""}
					>
						<option value="">Select a model...</option>
						{models.map((model) => (
							<option key={model.path} value={model.path}>
								{model.name} ({formatFileSize(model.size)})
							</option>
						))}
					</select>
				</div>
			)}
		</div>
	)
}
