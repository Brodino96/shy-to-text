mod audio;
mod config;
mod state;
mod transcribe;

use audio::RecordingSession;
use config::{Config, ModelInfo};
use parking_lot::Mutex;
use state::{AppState, AppStateManager};
use transcribe::LanguageInfo;
use std::sync::Arc;
use tauri::{
	image::Image,
	menu::{Menu, MenuItem},
	tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
	AppHandle, Emitter, Manager,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;

static RECORDING_SESSION: Mutex<Option<RecordingSession>> = Mutex::new(None);

#[tauri::command]
fn get_app_state(state: tauri::State<Arc<AppStateManager>>) -> AppState {
	state.get_state()
}

#[tauri::command]
fn get_config(state: tauri::State<Arc<AppStateManager>>) -> Config {
	state.get_config()
}

#[tauri::command]
fn save_config(
	app: AppHandle,
	state: tauri::State<Arc<AppStateManager>>,
	config: Config,
) -> Result<(), String> {
	let old_config = state.get_config();
	state.update_config(config.clone()).map_err(|e| e.to_string())?;

	if old_config.hotkey != config.hotkey {
		let _ = app.global_shortcut().unregister_all();
		setup_global_shortcut(&app, &config.hotkey)?;
	}

	Ok(())
}

#[tauri::command]
fn get_available_models() -> Result<Vec<ModelInfo>, String> {
	Config::detect_models().map_err(|e| e.to_string())
}

#[tauri::command]
fn load_model(
	state: tauri::State<Arc<AppStateManager>>,
	model_path: String,
) -> Result<(), String> {
	state.load_model(&model_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn has_model_loaded(state: tauri::State<Arc<AppStateManager>>) -> bool {
	state.has_model()
}

#[tauri::command]
fn is_model_multilingual(state: tauri::State<Arc<AppStateManager>>) -> bool {
	state.is_multilingual()
}

#[tauri::command]
fn get_last_transcription(state: tauri::State<Arc<AppStateManager>>) -> String {
	state.get_last_transcription()
}

#[tauri::command]
fn get_last_error(state: tauri::State<Arc<AppStateManager>>) -> Option<String> {
	state.get_error()
}

#[tauri::command]
fn get_models_directory() -> Result<String, String> {
	Config::models_dir()
		.map(|p| p.to_string_lossy().to_string())
		.map_err(|e| e.to_string())
}

#[tauri::command]
fn get_input_devices() -> Result<Vec<String>, String> {
	audio::list_input_devices().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_supported_languages() -> Vec<LanguageInfo> {
	transcribe::get_supported_languages()
}

fn toggle_recording(app: &AppHandle) {
	let state = app.state::<Arc<AppStateManager>>();
	let current_state = state.get_state();

	match current_state {
		AppState::Idle => {
			if !state.has_model() {
				state.set_error(Some("No model loaded".to_string()));
				let _ = app.emit("error", "No model loaded. Please load a Whisper model first.");
				show_notification(app, "Error", "No model loaded");
				return;
			}

			match RecordingSession::start() {
				Ok(session) => {
					*RECORDING_SESSION.lock() = Some(session);
					state.set_state(AppState::Recording);
					state.set_error(None);
					let _ = app.emit("state-changed", AppState::Recording);
					update_tray_tooltip(app, "Recording...");
				}
				Err(e) => {
					state.set_error(Some(e.to_string()));
					let _ = app.emit("error", e.to_string());
					show_notification(app, "Error", &format!("Failed to start recording: {}", e));
				}
			}
		}
		AppState::Recording => {
			state.set_state(AppState::Transcribing);
			let _ = app.emit("state-changed", AppState::Transcribing);
			update_tray_tooltip(app, "Transcribing...");

			let session = RECORDING_SESSION.lock().take();

			if let Some(session) = session {
				match session.stop() {
					Ok(samples) => {
						let app_clone = app.clone();
						std::thread::spawn(move || {
							process_transcription(&app_clone, samples);
						});
					}
					Err(e) => {
						state.set_state(AppState::Idle);
						state.set_error(Some(e.to_string()));
						let _ = app.emit("state-changed", AppState::Idle);
						let _ = app.emit("error", e.to_string());
						show_notification(app, "Error", &format!("Recording failed: {}", e));
						update_tray_tooltip(app, "Idle - Press F9 to record");
					}
				}
			} else {
				state.set_state(AppState::Idle);
				let _ = app.emit("state-changed", AppState::Idle);
				update_tray_tooltip(app, "Idle - Press F9 to record");
			}
		}
		AppState::Transcribing => {}
	}
}

fn process_transcription(app: &AppHandle, samples: Vec<f32>) {
	let state = app.state::<Arc<AppStateManager>>();
	let config = state.get_config();

	let language = if config.language == "auto" {
		None
	} else {
		Some(config.language.as_str())
	};

	let result = {
		let transcriber = state.transcriber.lock();
		if let Some(ref t) = *transcriber {
			t.transcribe(&samples, language)
		} else {
			Err(anyhow::anyhow!("No model loaded"))
		}
	};

	match result {
		Ok(text) => {
			if !text.is_empty() {
				state.set_last_transcription(text.clone());

				let config = state.get_config();
				if config.auto_copy {
					let _ = app.clipboard().write_text(&text);
				}

				let _ = app.emit("transcription", &text);

				if config.show_notifications {
					let preview = if text.len() > 50 {
						format!("{}...", &text[..50])
					} else {
						text.clone()
					};
					show_notification(app, "Transcribed", &preview);
				}
			} else {
				show_notification(
					app,
					"No speech detected",
					"Try speaking louder or closer to the microphone",
				);
			}

			state.set_error(None);
		}
		Err(e) => {
			state.set_error(Some(e.to_string()));
			let _ = app.emit("error", e.to_string());
			show_notification(app, "Transcription failed", &e.to_string());
		}
	}

	state.set_state(AppState::Idle);
	let _ = app.emit("state-changed", AppState::Idle);
	update_tray_tooltip(app, "Idle - Press F9 to record");
}

fn show_notification(app: &AppHandle, title: &str, body: &str) {
	let _ = app.notification().builder().title(title).body(body).show();
}

fn update_tray_tooltip(app: &AppHandle, tooltip: &str) {
	if let Some(tray) = app.tray_by_id("main-tray") {
		let _ = tray.set_tooltip(Some(tooltip));
	}
}

fn parse_hotkey(hotkey: &str) -> Option<Shortcut> {
	let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
	let mut modifiers = Modifiers::empty();
	let mut key_code = None;

	for part in parts {
		match part.to_uppercase().as_str() {
			"CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
			"ALT" => modifiers |= Modifiers::ALT,
			"SHIFT" => modifiers |= Modifiers::SHIFT,
			"SUPER" | "META" | "WIN" => modifiers |= Modifiers::SUPER,
			"F1" => key_code = Some(Code::F1),
			"F2" => key_code = Some(Code::F2),
			"F3" => key_code = Some(Code::F3),
			"F4" => key_code = Some(Code::F4),
			"F5" => key_code = Some(Code::F5),
			"F6" => key_code = Some(Code::F6),
			"F7" => key_code = Some(Code::F7),
			"F8" => key_code = Some(Code::F8),
			"F9" => key_code = Some(Code::F9),
			"F10" => key_code = Some(Code::F10),
			"F11" => key_code = Some(Code::F11),
			"F12" => key_code = Some(Code::F12),
			"A" => key_code = Some(Code::KeyA),
			"B" => key_code = Some(Code::KeyB),
			"C" => key_code = Some(Code::KeyC),
			"D" => key_code = Some(Code::KeyD),
			"E" => key_code = Some(Code::KeyE),
			"F" => key_code = Some(Code::KeyF),
			"G" => key_code = Some(Code::KeyG),
			"H" => key_code = Some(Code::KeyH),
			"I" => key_code = Some(Code::KeyI),
			"J" => key_code = Some(Code::KeyJ),
			"K" => key_code = Some(Code::KeyK),
			"L" => key_code = Some(Code::KeyL),
			"M" => key_code = Some(Code::KeyM),
			"N" => key_code = Some(Code::KeyN),
			"O" => key_code = Some(Code::KeyO),
			"P" => key_code = Some(Code::KeyP),
			"Q" => key_code = Some(Code::KeyQ),
			"R" => key_code = Some(Code::KeyR),
			"S" => key_code = Some(Code::KeyS),
			"T" => key_code = Some(Code::KeyT),
			"U" => key_code = Some(Code::KeyU),
			"V" => key_code = Some(Code::KeyV),
			"W" => key_code = Some(Code::KeyW),
			"X" => key_code = Some(Code::KeyX),
			"Y" => key_code = Some(Code::KeyY),
			"Z" => key_code = Some(Code::KeyZ),
			"0" => key_code = Some(Code::Digit0),
			"1" => key_code = Some(Code::Digit1),
			"2" => key_code = Some(Code::Digit2),
			"3" => key_code = Some(Code::Digit3),
			"4" => key_code = Some(Code::Digit4),
			"5" => key_code = Some(Code::Digit5),
			"6" => key_code = Some(Code::Digit6),
			"7" => key_code = Some(Code::Digit7),
			"8" => key_code = Some(Code::Digit8),
			"9" => key_code = Some(Code::Digit9),
			"SPACE" => key_code = Some(Code::Space),
			_ => {}
		}
	}

	key_code.map(|code| {
		if modifiers.is_empty() {
			Shortcut::new(None, code)
		} else {
			Shortcut::new(Some(modifiers), code)
		}
	})
}

fn setup_global_shortcut(app: &AppHandle, hotkey: &str) -> Result<(), String> {
	let shortcut = parse_hotkey(hotkey).ok_or_else(|| format!("Invalid hotkey: {}", hotkey))?;

	let app_clone = app.clone();
	app.global_shortcut()
		.on_shortcut(shortcut, move |_app, _shortcut, event| {
			if event.state == ShortcutState::Pressed {
				toggle_recording(&app_clone);
			}
		})
		.map_err(|e| e.to_string())?;

	Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_global_shortcut::Builder::new().build())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_clipboard_manager::init())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_fs::init())
		.setup(|app| {
			let state_manager = AppStateManager::new();
			let config = state_manager.get_config();

			app.manage(state_manager);

			let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
			let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
			let menu = Menu::with_items(app, &[&show, &quit])?;

			let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
				Image::new(&[], 1, 1)
			});

			let _tray = TrayIconBuilder::with_id("main-tray")
				.icon(icon)
				.icon_as_template(true)
				.menu(&menu)
				.tooltip("Shy to Text - Press F9 to record")
				.on_menu_event(|app, event| match event.id.as_ref() {
					"quit" => {
						app.exit(0);
					}
					"show" => {
						if let Some(window) = app.get_webview_window("main") {
							let _ = window.show();
							let _ = window.set_focus();
						}
					}
					_ => {}
				})
				.on_tray_icon_event(|tray, event| {
					if let TrayIconEvent::Click {
						button: MouseButton::Left,
						button_state: MouseButtonState::Up,
						..
					} = event
					{
						let app = tray.app_handle();
						if let Some(window) = app.get_webview_window("main") {
							let _ = window.show();
							let _ = window.set_focus();
						}
					}
				})
				.build(app)?;

			let app_handle = app.handle().clone();
			if let Err(e) = setup_global_shortcut(&app_handle, &config.hotkey) {
				eprintln!("Failed to setup global shortcut: {}", e);
			}

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			get_app_state,
			get_config,
			save_config,
			get_available_models,
			load_model,
			has_model_loaded,
			is_model_multilingual,
			get_last_transcription,
			get_last_error,
			get_models_directory,
			get_input_devices,
			get_supported_languages,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
