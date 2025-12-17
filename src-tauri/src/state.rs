use crate::config::Config;
use crate::transcribe::Transcriber;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppState {
	Idle,
	Recording,
	Transcribing,
}

pub struct AppStateManager {
	pub state: Mutex<AppState>,
	pub config: Mutex<Config>,
	pub transcriber: Mutex<Option<Transcriber>>,
	pub last_transcription: Mutex<String>,
	pub error: Mutex<Option<String>>,
}

unsafe impl Send for AppStateManager {}
unsafe impl Sync for AppStateManager {}

impl AppStateManager {
	pub fn new() -> Arc<Self> {
		let mut config = Config::load().unwrap_or_default();

		let transcriber = if let Some(ref model_path) = config.model_path {
			match Transcriber::new(model_path, config.use_gpu, config.gpu_device) {
				Ok(result) => {
					if result.gpu_fallback {
						config.use_gpu = false;
						let _ = config.save();
					}
					Some(result.transcriber)
				}
				Err(_) => None,
			}
		} else {
			None
		};

		Arc::new(Self {
			state: Mutex::new(AppState::Idle),
			config: Mutex::new(config),
			transcriber: Mutex::new(transcriber),
			last_transcription: Mutex::new(String::new()),
			error: Mutex::new(None),
		})
	}

	pub fn get_state(&self) -> AppState {
		*self.state.lock()
	}

	pub fn set_state(&self, state: AppState) {
		*self.state.lock() = state;
	}

	pub fn get_config(&self) -> Config {
		self.config.lock().clone()
	}

	pub fn update_config(&self, config: Config) -> anyhow::Result<()> {
		config.save()?;
		*self.config.lock() = config;
		Ok(())
	}

	/// Loads a model with the current GPU configuration.
	/// Returns true if GPU fallback to CPU occurred.
	pub fn load_model(&self, model_path: &str) -> anyhow::Result<bool> {
		let (use_gpu, gpu_device) = {
			let config = self.config.lock();
			(config.use_gpu, config.gpu_device)
		};

		let result = Transcriber::new(model_path, use_gpu, gpu_device)?;
		*self.transcriber.lock() = Some(result.transcriber);

		let mut config = self.config.lock();
		config.model_path = Some(model_path.to_string());
		if result.gpu_fallback {
			config.use_gpu = false;
		}
		config.save()?;

		Ok(result.gpu_fallback)
	}

	/// Reloads the currently loaded model with updated GPU configuration.
	/// Returns true if GPU fallback to CPU occurred.
	pub fn reload_model(&self) -> anyhow::Result<bool> {
		let (model_path, use_gpu, gpu_device) = {
			let config = self.config.lock();
			let model_path = config
				.model_path
				.clone()
				.ok_or_else(|| anyhow::anyhow!("No model loaded"))?;
			(model_path, config.use_gpu, config.gpu_device)
		};

		let result = Transcriber::new(&model_path, use_gpu, gpu_device)?;
		*self.transcriber.lock() = Some(result.transcriber);

		if result.gpu_fallback {
			let mut config = self.config.lock();
			config.use_gpu = false;
			config.save()?;
		}

		Ok(result.gpu_fallback)
	}

	pub fn has_model(&self) -> bool {
		self.transcriber.lock().is_some()
	}

	pub fn is_multilingual(&self) -> bool {
		self.transcriber
			.lock()
			.as_ref()
			.map(|t| t.is_multilingual())
			.unwrap_or(false)
	}

	pub fn set_error(&self, error: Option<String>) {
		*self.error.lock() = error;
	}

	pub fn get_error(&self) -> Option<String> {
		self.error.lock().clone()
	}

	pub fn set_last_transcription(&self, text: String) {
		*self.last_transcription.lock() = text;
	}

	pub fn get_last_transcription(&self) -> String {
		self.last_transcription.lock().clone()
	}
}
