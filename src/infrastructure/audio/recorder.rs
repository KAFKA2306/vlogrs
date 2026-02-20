use crate::domain::AudioRecorder as AudioRecorderTrait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracing::{error, info};
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
        let part_path: PathBuf =
            output_path.with_extension(crate::domain::constants::WAV_PART_EXTENSION);
        let host: cpal::Host = cpal::default_host();
        let device: cpal::Device = match device_name {
            Some(name) => host
                .input_devices()
                .unwrap()
                .find(|d| d.name().map(|n| n.contains(&name)).unwrap_or(false))
                .unwrap_or_else(|| host.default_input_device().unwrap()),
            None => host.default_input_device().unwrap(),
        };
        let supported_configs = device.supported_input_configs().unwrap();
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
            .unwrap_or_else(|| device.default_input_config().unwrap().config());
        let sample_format: cpal::SampleFormat = device
            .supported_input_configs()
            .unwrap()
            .find(|c| {
                c.channels() == config.channels
                    && c.min_sample_rate().0 <= config.sample_rate.0
                    && c.max_sample_rate().0 >= config.sample_rate.0
            })
            .map(|c| c.sample_format())
            .unwrap_or(cpal::SampleFormat::F32);
        let spec: hound::WavSpec = hound::WavSpec {
            channels: config.channels,
            sample_rate: config.sample_rate.0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let writer: Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>> =
            Arc::new(Mutex::new(Some(
                hound::WavWriter::create(&part_path, spec).unwrap(),
            )));
        let writer_cb = writer.clone();
        let last_log: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
        let peak_amplitude: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0f32));
        let stream: cpal::Stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
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
                )
                .unwrap(),
            cpal::SampleFormat::I16 => device
                .build_input_stream(
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
                )
                .unwrap(),
            _ => panic!("Unsupported sample format: {:?}", sample_format),
        };
        stream.play().unwrap();
        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(
                crate::domain::constants::AUDIO_SLEEP_MS,
            ));
        }
        drop(stream);
        let mut guard = writer.lock().unwrap();
        if let Some(w) = guard.take() {
            w.finalize().unwrap();
        }
        Ok(())
    }
    fn process_audio<I>(
        samples: I,
        writer_cb: &Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>,
        _silence_threshold: f32,
        peak_amplitude: &Arc<Mutex<f32>>,
        last_log: &Arc<Mutex<Instant>>,
    ) where
        I: Iterator<Item = f32>,
    {
        let mut local_peak: f32 = 0.0f32;
        let mut samples_to_write: Vec<i16> = Vec::new();
        for sample in samples {
            let abs_sample: f32 = sample.abs();
            if abs_sample > local_peak {
                local_peak = abs_sample;
            }
            let s: i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            samples_to_write.push(s);
        }
        if !samples_to_write.is_empty() {
            let mut guard = writer_cb.lock().unwrap();
            if let Some(w) = guard.as_mut() {
                for s in samples_to_write {
                    w.write_sample(s).unwrap();
                }
            }
        }
        let mut peak = peak_amplitude.lock().unwrap();
        if local_peak > *peak {
            *peak = local_peak;
        }
        let mut last = last_log.lock().unwrap();
        if last.elapsed() >= Duration::from_secs(crate::domain::constants::AUDIO_LOG_INTERVAL_SECS)
        {
            let p = peak_amplitude.lock().unwrap();
            info!("Recording status: peak_amplitude={:.4}", *p);
            *last = Instant::now();
            drop(p);
            let mut p_reset = peak_amplitude.lock().unwrap();
            *p_reset = 0.0;
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
        let mut current_file = self.current_file.lock().unwrap();
        *current_file = Some(output_path.clone());
        let is_recording: Arc<AtomicBool> = self.is_recording.clone();
        let handle: JoinHandle<()> = thread::spawn(move || {
            let part_path: PathBuf =
                output_path.with_extension(crate::domain::constants::WAV_PART_EXTENSION);
            Self::record_loop(
                part_path,
                sample_rate,
                channels,
                is_recording,
                device_name,
                silence_threshold,
            )
            .unwrap();
        });
        let mut recording_thread = self.recording_thread.lock().unwrap();
        *recording_thread = Some(handle);
    }
    fn stop(&self) -> Option<PathBuf> {
        self.is_recording.store(false, Ordering::SeqCst);
        let join_handle = {
            let mut recording_thread = self.recording_thread.lock().unwrap();
            recording_thread.take()
        };
        if let Some(handle) = join_handle {
            handle.join().unwrap();
        }
        let mut current_file = self.current_file.lock().unwrap();
        let path: Option<PathBuf> = current_file.take();
        if let Some(ref p) = path {
            let part_path: PathBuf = p.with_extension(crate::domain::constants::WAV_PART_EXTENSION);
            if part_path.exists() {
                std::fs::rename(&part_path, p).unwrap();
                if let Some(parent) = p.parent() {
                    let dir: std::fs::File = std::fs::File::open(parent).unwrap();
                    dir.sync_all().unwrap();
                }
            }
        }
        path
    }
}
