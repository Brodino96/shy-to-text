use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use parking_lot::Mutex;
use rubato::{FftFixedIn, Resampler};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct RecordingSession {
	samples: Arc<Mutex<Vec<f32>>>,
	sample_rate: u32,
	is_recording: Arc<AtomicBool>,
}

impl RecordingSession {
	pub fn start() -> Result<Self> {
		let host = cpal::default_host();
		let device = host
			.default_input_device()
			.context("No input device available")?;

		let config = device
			.default_input_config()
			.context("Failed to get default input config")?;

		let sample_rate = config.sample_rate().0;
		let channels = config.channels() as usize;

		let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
		let is_recording = Arc::new(AtomicBool::new(true));

		let samples_clone = Arc::clone(&samples);
		let is_recording_clone = Arc::clone(&is_recording);

		let err_fn = |err| eprintln!("Audio stream error: {}", err);

		let stream = match config.sample_format() {
			SampleFormat::F32 => device.build_input_stream(
				&config.into(),
				move |data: &[f32], _: &_| {
					if is_recording_clone.load(Ordering::SeqCst) {
						let mono: Vec<f32> = if channels > 1 {
							data.chunks(channels)
								.map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
								.collect()
						} else {
							data.to_vec()
						};
						samples_clone.lock().extend(mono);
					}
				},
				err_fn,
				None,
			)?,
			SampleFormat::I16 => {
				let samples_clone = Arc::clone(&samples);
				let is_recording_clone = Arc::clone(&is_recording);
				device.build_input_stream(
					&config.into(),
					move |data: &[i16], _: &_| {
						if is_recording_clone.load(Ordering::SeqCst) {
							let mono: Vec<f32> = if channels > 1 {
								data.chunks(channels)
									.map(|chunk| {
										chunk.iter().map(|&s| s.to_float_sample()).sum::<f32>()
											/ channels as f32
									})
									.collect()
							} else {
								data.iter().map(|&s| s.to_float_sample()).collect()
							};
							samples_clone.lock().extend(mono);
						}
					},
					err_fn,
					None,
				)?
			}
			SampleFormat::U16 => {
				let samples_clone = Arc::clone(&samples);
				let is_recording_clone = Arc::clone(&is_recording);
				device.build_input_stream(
					&config.into(),
					move |data: &[u16], _: &_| {
						if is_recording_clone.load(Ordering::SeqCst) {
							let mono: Vec<f32> = if channels > 1 {
								data.chunks(channels)
									.map(|chunk| {
										chunk.iter().map(|&s| s.to_float_sample()).sum::<f32>()
											/ channels as f32
									})
									.collect()
							} else {
								data.iter().map(|&s| s.to_float_sample()).collect()
							};
							samples_clone.lock().extend(mono);
						}
					},
					err_fn,
					None,
				)?
			}
			_ => anyhow::bail!("Unsupported sample format"),
		};

		stream.play().context("Failed to start audio stream")?;

		std::mem::forget(stream);

		Ok(Self {
			samples,
			sample_rate,
			is_recording,
		})
	}

	pub fn stop(self) -> Result<Vec<f32>> {
		self.is_recording.store(false, Ordering::SeqCst);

		std::thread::sleep(std::time::Duration::from_millis(100));

		let samples = self.samples.lock().clone();

		if samples.is_empty() {
			anyhow::bail!("No audio recorded");
		}

		resample_to_16khz(&samples, self.sample_rate)
	}
}

fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Result<Vec<f32>> {
	const TARGET_RATE: u32 = 16000;

	if source_rate == TARGET_RATE {
		return Ok(samples.to_vec());
	}

	let mut resampler = FftFixedIn::<f32>::new(source_rate as usize, TARGET_RATE as usize, 1024, 2, 1)
		.context("Failed to create resampler")?;

	let mut output = Vec::new();
	let input_frames_needed = resampler.input_frames_next();

	for chunk in samples.chunks(input_frames_needed) {
		let input = if chunk.len() < input_frames_needed {
			let mut padded = chunk.to_vec();
			padded.resize(input_frames_needed, 0.0);
			vec![padded]
		} else {
			vec![chunk.to_vec()]
		};

		let resampled = resampler.process(&input, None)?;
		if !resampled.is_empty() {
			output.extend(&resampled[0]);
		}
	}

	Ok(output)
}

pub fn list_input_devices() -> Result<Vec<String>> {
	let host = cpal::default_host();
	let devices: Vec<String> = host
		.input_devices()?
		.filter_map(|d| d.name().ok())
		.collect();
	Ok(devices)
}
