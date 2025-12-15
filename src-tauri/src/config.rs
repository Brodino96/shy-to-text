use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub hotkey: String,
	pub language: String,
	pub model_path: Option<String>,
	pub auto_copy: bool,
	pub show_notifications: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			hotkey: "F9".to_string(),
			language: "auto".to_string(),
			model_path: None,
			auto_copy: true,
			show_notifications: true,
		}
	}
}

impl Config {
	pub fn config_dir() -> Result<PathBuf> {
		let config_dir = dirs::config_dir()
			.context("Failed to get config directory")?
			.join("shy-to-text");

		if !config_dir.exists() {
			fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
		}

		Ok(config_dir)
	}

	pub fn models_dir() -> Result<PathBuf> {
		let models_dir = Self::config_dir()?.join("models");

		if !models_dir.exists() {
			fs::create_dir_all(&models_dir).context("Failed to create models directory")?;
		}

		Ok(models_dir)
	}

	pub fn config_path() -> Result<PathBuf> {
		Ok(Self::config_dir()?.join("config.json"))
	}

	pub fn load() -> Result<Self> {
		let config_path = Self::config_path()?;

		if config_path.exists() {
			let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
			let config: Config =
				serde_json::from_str(&content).context("Failed to parse config file")?;
			Ok(config)
		} else {
			let config = Config::default();
			config.save()?;
			Ok(config)
		}
	}

	pub fn save(&self) -> Result<()> {
		let config_path = Self::config_path()?;
		let content =
			serde_json::to_string_pretty(self).context("Failed to serialize config")?;
		fs::write(&config_path, content).context("Failed to write config file")?;
		Ok(())
	}

	pub fn detect_models() -> Result<Vec<ModelInfo>> {
		let models_dir = Self::models_dir()?;
		let mut models = Vec::new();

		if models_dir.exists() {
			for entry in fs::read_dir(&models_dir)? {
				let entry = entry?;
				let path = entry.path();
				if path.extension().map_or(false, |ext| ext == "bin") {
					if let Some(name) = path.file_stem() {
						models.push(ModelInfo {
							name: name.to_string_lossy().to_string(),
							path: path.to_string_lossy().to_string(),
							size: entry.metadata().map(|m| m.len()).unwrap_or(0),
						});
					}
				}
			}
		}

		Ok(models)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
	pub name: String,
	pub path: String,
	pub size: u64,
}
