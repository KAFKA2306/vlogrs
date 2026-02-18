use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tracing::{error, info, warn};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::{Result, Context};

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    current_file: Arc<Mutex<Option<PathBuf>>>,
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
        }
    }

    pub fn start(&self, output_path: impl Into<PathBuf>, sample_rate: u32, channels: u16, device_name: Option<String>, silence_threshold: f32) -> Result<()> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        let output_path = output_path.into();
        self.is_recording.store(true, Ordering::SeqCst);
        let mut current_file = self.current_file.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current_file"))?;
        *current_file = Some(output_path.clone());

        let is_recording = self.is_recording.clone();

        thread::spawn(move || {
            let part_path = output_path.with_extension("wav.part");
            if let Err(e) = Self::record_loop(part_path, sample_rate, channels, is_recording, device_name, silence_threshold) {
                error!("Audio recording loop failed: {}", e);
            }
        });

        info!("Started recording...");
        Ok(())
    }

    pub fn stop(&self) -> Result<Option<PathBuf>> {
        self.is_recording.store(false, Ordering::SeqCst);
        let mut current_file = self.current_file.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current_file"))?;
        let path = current_file.take();
        if let Some(ref p) = path {
            let part_path = p.with_extension("wav.part");
            if part_path.exists() {
                std::fs::rename(&part_path, p).context("Failed to rename part file")?;
                // Sync parent directory
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

    fn record_loop(path: PathBuf, sample_rate: u32, channels: u16, is_recording: Arc<AtomicBool>, device_name: Option<String>, silence_threshold: f32) -> Result<()> {
        let host = cpal::default_host();
        
        let device = match device_name {
            Some(name) => host.input_devices()
                .context("Failed to list input devices")?
                .find(|d| d.name().map(|n| n.contains(&name)).unwrap_or(false))
                .unwrap_or_else(|| {
                    warn!("Device '{}' not found, falling back to default.", name);
                    host.default_input_device().expect("No default input device found")
                }),
            None => host.default_input_device().context("No default input device found")?,
        };

        info!("Using audio device: {}", device.name().unwrap_or_default());

        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = Arc::new(Mutex::new(Some(
            hound::WavWriter::create(&path, spec).context("Failed to create wav writer")?,
        )));
        let writer_cb = writer.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer_cb.lock() {
                    if let Some(w) = guard.as_mut() {
                        for &sample in data {
                            if sample.abs() > silence_threshold {
                                let s = (sample * i16::MAX as f32) as i16;
                                let _ = w.write_sample(s);
                            }
                        }
                    }
                }
            },
            move |err| {
                error!("Audio stream error: {}", err);
            },
            None,
        ).context("Failed to build input stream")?;

        stream.play().context("Failed to start audio stream")?;

        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
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
            .arg("16000")
            .arg("-ac")
            .arg("1")
            .arg(output)
            .output()?;

        if !status.status.success() {
             let err = String::from_utf8_lossy(&status.stderr);
             error!("ffmpeg failed: {}", err);
             return Err(std::io::Error::new(std::io::ErrorKind::Other, "ffmpeg failed"));
        }
        Ok(())
    }
}
