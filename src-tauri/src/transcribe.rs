use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{
	get_lang_max_id, get_lang_str, get_lang_str_full, FullParams, SamplingStrategy, WhisperContext,
	WhisperContextParameters,
};

pub struct Transcriber {
	ctx: WhisperContext,
	is_multilingual: bool,
}

impl Transcriber {
	pub fn new(model_path: &str) -> Result<Self> {
		let path = Path::new(model_path);
		if !path.exists() {
			anyhow::bail!("Model file not found: {}", model_path);
		}

		let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
			.context("Failed to load Whisper model")?;

		let is_multilingual = ctx.is_multilingual();

		Ok(Self {
			ctx,
			is_multilingual,
		})
	}

	pub fn transcribe(&self, samples: &[f32], language: Option<&str>) -> Result<String> {
		let mut state = self.ctx.create_state().context("Failed to create state")?;

		let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

		params.set_print_special(false);
		params.set_print_progress(false);
		params.set_print_realtime(false);
		params.set_print_timestamps(false);
		params.set_suppress_blank(true);
		params.set_suppress_nst(true);
		params.set_translate(false);

		if self.is_multilingual {
			if let Some(lang) = language {
				if lang != "auto" {
					params.set_language(Some(lang));
				}
			}
		} else {
			params.set_language(Some("en"));
		}

		params.set_n_threads(num_cpus());

		state
			.full(params, samples)
			.context("Failed to run transcription")?;

		let num_segments = state.full_n_segments();
		let mut result = String::new();

		for i in 0..num_segments {
			if let Some(segment) = state.get_segment(i) {
				if let Ok(text) = segment.to_str_lossy() {
					result.push_str(&text);
				}
			}
		}

		Ok(result.trim().to_string())
	}

	pub fn is_multilingual(&self) -> bool {
		self.is_multilingual
	}
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LanguageInfo {
	pub code: String,
	pub name: String,
}

pub fn get_supported_languages() -> Vec<LanguageInfo> {
	let max_id = get_lang_max_id();
	let mut languages = Vec::with_capacity((max_id + 1) as usize);

	for id in 0..=max_id {
		if let (Some(code), Some(name)) = (get_lang_str(id), get_lang_str_full(id)) {
			let display_name = capitalize_first(name);
			languages.push(LanguageInfo {
				code: code.to_string(),
				name: display_name,
			});
		}
	}

	languages
}

fn capitalize_first(s: &str) -> String {
	let mut chars = s.chars();
	match chars.next() {
		None => String::new(),
		Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
	}
}

fn num_cpus() -> i32 {
	std::thread::available_parallelism()
		.map(|p| p.get() as i32)
		.unwrap_or(4)
		.min(8)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_capitalize_first_empty() {
		assert_eq!(capitalize_first(""), "");
	}

	#[test]
	fn test_capitalize_first_single_char() {
		assert_eq!(capitalize_first("a"), "A");
		assert_eq!(capitalize_first("Z"), "Z");
	}

	#[test]
	fn test_capitalize_first_already_capitalized() {
		assert_eq!(capitalize_first("English"), "English");
	}

	#[test]
	fn test_supported_languages_not_empty() {
		let languages = get_supported_languages();
		assert!(!languages.is_empty());
	}

	#[test]
	fn test_supported_languages_have_code_and_name() {
		let languages = get_supported_languages();

		for lang in &languages {
			assert!(!lang.code.is_empty(), "Language code should not be empty");
			assert!(!lang.name.is_empty(), "Language name should not be empty");
		}
	}

	#[test]
	fn test_supported_languages_contains_english() {
		let languages = get_supported_languages();
		let has_english = languages.iter().any(|l| l.code == "en");
		assert!(has_english, "Supported languages should include English");
	}
}
