use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    current_file: Arc<Mutex<Option<String>>>,
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

    pub fn start(
        &self,
        output_path: String,
        sample_rate: u32,
        channels: u16,
    ) -> anyhow::Result<()> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_recording.store(true, Ordering::SeqCst);
        *self.current_file.lock().unwrap() = Some(output_path.clone());

        let is_recording = self.is_recording.clone();

        thread::spawn(move || {
            if let Err(e) = Self::record_loop(output_path, sample_rate, channels, is_recording) {
                error!("Recording loop error: {}", e);
            }
        });

        info!("Started recording...");
        Ok(())
    }

    pub fn stop(&self) -> Option<String> {
        self.is_recording.store(false, Ordering::SeqCst);
        let path = self.current_file.lock().unwrap().take();
        if let Some(ref p) = path {
            info!("Stopped recording: {}", p);
        }
        path
    }

    fn record_loop(
        path: String,
        sample_rate: u32,
        channels: u16,
        is_recording: Arc<AtomicBool>,
    ) -> anyhow::Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::Error::msg("No input device found"))?;

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

        let writer = Arc::new(Mutex::new(Some(hound::WavWriter::create(&path, spec)?)));
        let writer_cb = writer.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Some(ref mut w) = *writer_cb.lock().unwrap() {
                    for &sample in data {
                        if sample.abs() > 0.005 {
                            let s = (sample * i16::MAX as f32) as i16;
                            w.write_sample(s).ok();
                        }
                    }
                }
            },
            move |err| {
                error!("Audio stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;

        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        drop(stream);
        if let Some(w) = writer.lock().unwrap().take() {
            w.finalize()?;
        }

        Ok(())
    }
}
