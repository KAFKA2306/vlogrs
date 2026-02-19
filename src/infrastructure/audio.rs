use crate::domain::AudioRecorder as AudioRecorderTrait;
use anyhow::{Context, Result};
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
        path: PathBuf,
        sample_rate: u32,
        channels: u16,
        is_recording: Arc<AtomicBool>,
        device_name: Option<String>,
        silence_threshold: f32,
    ) -> Result<()> {
        let host = cpal::default_host();

        let device = match device_name {
            Some(name) => host
                .input_devices()
                .context("Failed to list input devices")?
                .find(|d| d.name().map(|n| n.contains(&name)).unwrap_or(false))
                .ok_or_else(|| {
                    warn!("Device '{}' not found, falling back to default.", name);
                    anyhow::anyhow!("Device '{}' not found", name)
                })
                .or_else(|_| {
                    host.default_input_device()
                        .context("No default input device found")
                })?,
            None => host
                .default_input_device()
                .context("No default input device found")?,
        };

        info!("Using audio device: {}", device.name().unwrap_or_default());

        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Strict Check: Ensure the device supports the exact configuration
        let supported = device.supported_input_configs()?.any(|c| {
            c.channels() == channels
                && c.min_sample_rate().0 <= sample_rate
                && c.max_sample_rate().0 >= sample_rate
                && c.sample_format() == cpal::SampleFormat::F32
        });

        if !supported {
            error!(
                "CRITICAL: Hardware does not natively support {}Hz {}ch F32.",
                sample_rate, channels
            );
            Self::list_devices()?;
            anyhow::bail!(
                "Hardware incompatibility: {}Hz {}ch unsupported",
                sample_rate,
                channels
            );
        }

        info!("Selected audio config: {:?}", config);

        let spec = hound::WavSpec {
            channels: config.channels,
            sample_rate: config.sample_rate.0,
            bits_per_sample: crate::domain::constants::DEFAULT_BITS_PER_SAMPLE,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = Arc::new(Mutex::new(Some(
            hound::WavWriter::create(&path, spec).context("Failed to create wav writer")?,
        )));
        let writer_cb = writer.clone();

        let last_log = Arc::new(Mutex::new(Instant::now()));
        let peak_amplitude = Arc::new(Mutex::new(0.0f32));

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut local_peak = 0.0f32;
                    if let Ok(mut guard) = writer_cb.lock() {
                        if let Some(w) = guard.as_mut() {
                            for &sample in data {
                                let abs_sample = sample.abs();
                                if abs_sample > local_peak {
                                    local_peak = abs_sample;
                                }

                                if abs_sample >= silence_threshold {
                                    let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                                    let _ = w.write_sample(s);
                                }
                            }
                        }
                    }

                    // Update global peak
                    if let Ok(mut p) = peak_amplitude.lock() {
                        if local_peak > *p {
                            *p = local_peak;
                        }
                    }

                    // Log peak level every interval
                    if let Ok(mut last) = last_log.lock() {
                        if last.elapsed()
                            >= Duration::from_secs(
                                crate::domain::constants::AUDIO_LOG_INTERVAL_SECS,
                            )
                        {
                            if let Ok(p) = peak_amplitude.lock() {
                                info!("Recording status: peak_amplitude={:.4}", *p);
                            }
                            *last = Instant::now();
                            // Reset peak for next interval
                            if let Ok(mut p) = peak_amplitude.lock() {
                                *p = 0.0;
                            }
                        }
                    }
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                },
                None,
            )
            .context("Failed to build input stream")?;

        stream.play().context("Failed to start audio stream")?;

        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(
                crate::domain::constants::AUDIO_SLEEP_MS,
            ));
        }

        drop(stream);
        if let Ok(mut guard) = writer.lock() {
            if let Some(w) = guard.take() {
                w.finalize().context("Failed to finalize wav file")?;
            }
        }
        Ok(())
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
            .arg(output)
            .output()?;

        if !status.status.success() {
            let err = String::from_utf8_lossy(&status.stderr);
            error!("ffmpeg failed: {}", err);
            return Err(std::io::Error::other("ffmpeg failed"));
        }
        Ok(())
    }

    pub fn list_devices() -> Result<()> {
        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .context("Failed to list input devices")?;

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
        Ok(())
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
    ) -> Result<()> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_recording.store(true, Ordering::SeqCst);
        let mut current_file = self
            .current_file
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current_file"))?;
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
        let mut recording_thread = self
            .recording_thread
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock recording_thread"))?;
        *recording_thread = Some(handle);

        info!("Started recording...");
        Ok(())
    }

    fn stop(&self) -> Result<Option<PathBuf>> {
        self.is_recording.store(false, Ordering::SeqCst);
        let join_handle = {
            let mut recording_thread = self
                .recording_thread
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock recording_thread"))?;
            recording_thread.take()
        };

        if let Some(handle) = join_handle {
            let _ = handle.join();
        }

        let mut current_file = self
            .current_file
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current_file"))?;
        let path = current_file.take();
        if let Some(ref p) = path {
            let part_path = p.with_extension(crate::domain::constants::WAV_PART_EXTENSION);
            if part_path.exists() {
                std::fs::rename(&part_path, p).context("Failed to rename part file")?;
                if let Some(parent) = p.parent() {
                    if let Ok(dir) = std::fs::File::open(parent) {
                        let _ = dir.sync_all();
                    }
                }
            }
            info!("Stopped recording: {:?}", p);
        }
        Ok(path)
    }
}
