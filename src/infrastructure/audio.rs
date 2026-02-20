use crate::domain::AudioRecorder as AudioRecorderTrait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    current_file: Arc<Mutex<Option<PathBuf>>>,
    recording_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            current_file: Arc::new(Mutex::new(None)),
            recording_thread: Arc::new(Mutex::new(None)),
        }
    }

    fn record_loop(
        output_path: PathBuf,
        sample_rate: u32,
        channels: u16,
        is_recording: Arc<AtomicBool>,
        device_name: Option<String>,
        silence_threshold: f32,
    ) -> anyhow::Result<()> {
        info!(
            "Audio recording loop started. Target path: {:?}",
            output_path
        );
        let part_path = output_path.with_extension(crate::domain::constants::WAV_PART_EXTENSION);
        let host = cpal::default_host();

        let device = match device_name {
            Some(name) => host
                .input_devices()
                .unwrap()
                .find(|d| d.name().map(|n| n.contains(&name)).unwrap_or(false))
                .unwrap_or_else(|| host.default_input_device().unwrap()),
            None => host.default_input_device().unwrap(),
        };

        info!("Using audio device: {}", device.name().unwrap_or_default());

        // Adaptive Config Selection
        let supported_configs = device.supported_input_configs()?;
        let config = supported_configs
            .filter(|c| {
                c.sample_format() == cpal::SampleFormat::F32
                    || c.sample_format() == cpal::SampleFormat::I16
            })
            .find(|c| {
                c.channels() == channels
                    && c.min_sample_rate().0 <= sample_rate
                    && c.max_sample_rate().0 >= sample_rate
            })
            .map(|c| c.with_sample_rate(cpal::SampleRate(sample_rate)).config())
            .unwrap_or_else(|| {
                warn!(
                    "Requested config ({}Hz, {}ch) unsupported. Using default.",
                    sample_rate, channels
                );
                device.default_input_config().unwrap().config()
            });

        let sample_format = device
            .supported_input_configs()?
            .find(|c| {
                c.channels() == config.channels
                    && c.min_sample_rate().0 <= config.sample_rate.0
                    && c.max_sample_rate().0 >= config.sample_rate.0
            })
            .map(|c| c.sample_format())
            .unwrap_or(cpal::SampleFormat::F32);

        info!("Selected audio config: {:?}", config);
        info!("Sample format: {:?}", sample_format);

        let spec = hound::WavSpec {
            channels: config.channels,
            sample_rate: config.sample_rate.0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = Arc::new(Mutex::new(Some(
            hound::WavWriter::create(&part_path, spec).unwrap(),
        )));
        let writer_cb = writer.clone();

        let last_log = Arc::new(Mutex::new(Instant::now()));
        let peak_amplitude = Arc::new(Mutex::new(0.0f32));

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    Self::process_audio(
                        data.iter().cloned(),
                        &writer_cb,
                        silence_threshold,
                        &peak_amplitude,
                        &last_log,
                    );
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let iter = data.iter().map(|&s| s as f32 / i16::MAX as f32);
                    Self::process_audio(
                        iter,
                        &writer_cb,
                        silence_threshold,
                        &peak_amplitude,
                        &last_log,
                    );
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let iter = data
                        .iter()
                        .map(|&s| (s as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0));
                    Self::process_audio(
                        iter,
                        &writer_cb,
                        silence_threshold,
                        &peak_amplitude,
                        &last_log,
                    );
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?,
            cpal::SampleFormat::I8 => device.build_input_stream(
                &config,
                move |data: &[i8], _: &cpal::InputCallbackInfo| {
                    let iter = data.iter().map(|&s| s as f32 / i8::MAX as f32);
                    Self::process_audio(
                        iter,
                        &writer_cb,
                        silence_threshold,
                        &peak_amplitude,
                        &last_log,
                    );
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?,
            cpal::SampleFormat::U8 => device.build_input_stream(
                &config,
                move |data: &[u8], _: &cpal::InputCallbackInfo| {
                    let iter = data.iter().map(|&s| (s as f32 - 128.0) / 128.0);
                    Self::process_audio(
                        iter,
                        &writer_cb,
                        silence_threshold,
                        &peak_amplitude,
                        &last_log,
                    );
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?,
            cpal::SampleFormat::I32 => device.build_input_stream(
                &config,
                move |data: &[i32], _: &cpal::InputCallbackInfo| {
                    let iter = data.iter().map(|&s| s as f32 / i32::MAX as f32);
                    Self::process_audio(
                        iter,
                        &writer_cb,
                        silence_threshold,
                        &peak_amplitude,
                        &last_log,
                    );
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?,
            cpal::SampleFormat::U32 => device.build_input_stream(
                &config,
                move |data: &[u32], _: &cpal::InputCallbackInfo| {
                    let iter = data
                        .iter()
                        .map(|&s| (s as f32 - u32::MAX as f32 / 2.0) / (u32::MAX as f32 / 2.0));
                    Self::process_audio(
                        iter,
                        &writer_cb,
                        silence_threshold,
                        &peak_amplitude,
                        &last_log,
                    );
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?,
            _ => panic!("Unsupported sample format: {:?}", sample_format),
        };

        stream.play().unwrap();

        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(
                crate::domain::constants::AUDIO_SLEEP_MS,
            ));
        }

        drop(stream);
        if let Ok(mut guard) = writer.lock() {
            if let Some(w) = guard.take() {
                w.finalize().unwrap();
            }
        }
        Ok(())
    }

    fn process_audio<I>(
        samples: I,
        writer_cb: &Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>,
        silence_threshold: f32,
        peak_amplitude: &Arc<Mutex<f32>>,
        last_log: &Arc<Mutex<Instant>>,
    ) where
        I: Iterator<Item = f32>,
    {
        let mut local_peak = 0.0f32;
        let mut samples_to_write = Vec::new();

        for sample in samples {
            let abs_sample = sample.abs();
            if abs_sample > local_peak {
                local_peak = abs_sample;
            }

            if abs_sample >= silence_threshold {
                let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                samples_to_write.push(s);
            }
        }

        if !samples_to_write.is_empty() {
            if let Ok(mut guard) = writer_cb.lock() {
                if let Some(w) = guard.as_mut() {
                    for s in samples_to_write {
                        let _ = w.write_sample(s);
                    }
                }
            }
        }

        // Update peak amplitude
        if let Ok(mut peak) = peak_amplitude.lock() {
            if local_peak > *peak {
                *peak = local_peak;
            }
        }

        // Periodic logging
        if let Ok(mut last) = last_log.lock() {
            if last.elapsed()
                >= Duration::from_secs(crate::domain::constants::AUDIO_LOG_INTERVAL_SECS)
            {
                if let Ok(p) = peak_amplitude.lock() {
                    info!("Recording status: peak_amplitude={:.4}", *p);
                }
                *last = Instant::now();
                if let Ok(mut p) = peak_amplitude.lock() {
                    *p = 0.0;
                }
            }
        }
    }

    pub fn normalize_audio(input: &PathBuf, output: &PathBuf) -> std::io::Result<()> {
        let status = std::process::Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input)
            .arg("-ar")
            .arg(crate::domain::constants::TARGET_SAMPLE_RATE.to_string())
            .arg("-ac")
            .arg(crate::domain::constants::TARGET_CHANNELS.to_string())
            .arg("-c:a")
            .arg("libopus")
            .arg("-b:a")
            .arg(crate::domain::constants::OPUS_BITRATE.to_string())
            .arg(output)
            .output()?;

        if !status.status.success() {
            let err = String::from_utf8_lossy(&status.stderr);
            error!("ffmpeg failed: {}", err);
            return Err(std::io::Error::other("ffmpeg failed"));
        }
        Ok(())
    }

    pub fn list_devices() {
        let host = cpal::default_host();
        let devices = host.input_devices().expect("Failed to list input devices");

        info!("=== Available Audio Input Devices ===");
        for (i, device) in devices.enumerate() {
            let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            info!("Device {}: {}", i, name);

            if let Ok(mut configs) = device.supported_input_configs() {
                for config in configs.by_ref() {
                    info!(
                        "  - Supported: channels={}, min_sample_rate={}, max_sample_rate={}, buffer_size={:?}, sample_format={:?}",
                        config.channels(),
                        config.min_sample_rate().0,
                        config.max_sample_rate().0,
                        config.buffer_size(),
                        config.sample_format()
                    );
                }
            }
        }
    }
}

impl AudioRecorderTrait for AudioRecorder {
    fn start(
        &self,
        output_path: PathBuf,
        sample_rate: u32,
        channels: u16,
        device_name: Option<String>,
        silence_threshold: f32,
    ) {
        if self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        self.is_recording.store(true, Ordering::SeqCst);
        let mut current_file = self
            .current_file
            .lock()
            .expect("Failed to lock current_file");
        *current_file = Some(output_path.clone());

        let is_recording = self.is_recording.clone();

        let handle = thread::spawn(move || {
            let part_path =
                output_path.with_extension(crate::domain::constants::WAV_PART_EXTENSION);
            if let Err(e) = Self::record_loop(
                part_path,
                sample_rate,
                channels,
                is_recording,
                device_name,
                silence_threshold,
            ) {
                error!("Audio recording loop failed: {}", e);
            }
        });
        let mut recording_thread = self.recording_thread.lock().unwrap();
        *recording_thread = Some(handle);

        info!("Started recording...");
    }

    fn stop(&self) -> Option<PathBuf> {
        self.is_recording.store(false, Ordering::SeqCst);
        let join_handle = {
            let mut recording_thread = self.recording_thread.lock().unwrap();
            recording_thread.take()
        };

        if let Some(handle) = join_handle {
            let _ = handle.join();
        }

        let mut current_file = self
            .current_file
            .lock()
            .expect("Failed to lock current_file");
        let path = current_file.take();
        if let Some(ref p) = path {
            let part_path = p.with_extension(crate::domain::constants::WAV_PART_EXTENSION);
            if part_path.exists() {
                std::fs::rename(&part_path, p).unwrap();
                if let Some(parent) = p.parent() {
                    if let Ok(dir) = std::fs::File::open(parent) {
                        let _ = dir.sync_all();
                    }
                }
            }
            info!("Stopped recording: {:?}", p);
        }
        path
    }
}
