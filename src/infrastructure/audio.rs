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

    pub fn start(&self, output_path: String, sample_rate: u32, channels: u16, device_name: Option<String>) {
        if self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        self.is_recording.store(true, Ordering::SeqCst);
        *self.current_file.lock().unwrap() = Some(output_path.clone());

        let is_recording: Arc<AtomicBool> = self.is_recording.clone();

        thread::spawn(move || {
            Self::record_loop(output_path, sample_rate, channels, is_recording, device_name);
        });

        info!("Started recording...");
    }

    pub fn stop(&self) -> Option<String> {
        self.is_recording.store(false, Ordering::SeqCst);
        let path: Option<String> = self.current_file.lock().unwrap().take();
        if let Some(ref p) = path {
            info!("Stopped recording: {}", p);
        }
        path
    }

    fn record_loop(path: String, sample_rate: u32, channels: u16, is_recording: Arc<AtomicBool>, device_name: Option<String>) {
        let host: cpal::Host = cpal::default_host();
        
        let device: cpal::Device = if let Some(name) = device_name {
            host.input_devices()
                .unwrap()
                .find(|d| d.name().unwrap().contains(&name))
                .unwrap_or_else(|| {
                    error!("Device '{}' not found, falling back to default.", name);
                    host.default_input_device().unwrap()
                })
        } else {
            host.default_input_device().unwrap()
        };

        info!("Using audio device: {}", device.name().unwrap());

        let config: cpal::StreamConfig = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let spec: hound::WavSpec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer: Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>> =
            Arc::new(Mutex::new(Some(
                hound::WavWriter::create(&path, spec).unwrap(),
            )));
        let writer_cb: Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>> =
            writer.clone();

        let stream: cpal::Stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut guard = writer_cb.lock().unwrap();
                    if let Some(w) = guard.as_mut() {
                        for &sample in data {
                            if sample.abs() > 0.005 {
                                let s: i16 = (sample * i16::MAX as f32) as i16;
                                w.write_sample(s).unwrap();
                            }
                        }
                    }
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                    panic!("Audio stream error");
                },
                None,
            )
            .unwrap();

        stream.play().unwrap();

        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        drop(stream);
        let mut guard = writer.lock().unwrap();
        if let Some(w) = guard.take() {
            w.finalize().unwrap();
        }
    }
}
