use crate::domain::AudioRecorder as AudioRecorderTrait;
use crate::infrastructure;
use crate::infrastructure::settings::AudioRecordingSettings;
use chrono::Local;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    if !cfg!(target_os = "windows") {
        tracing::warn!("Recording is disabled on non-Windows hosts; skipping.");
        return Ok(());
    }
    info!("Starting audio recording...");
    let settings: AudioRecordingSettings =
        infrastructure::settings::Settings::get_audio_recording_settings()?;
    let recorder = Arc::new(infrastructure::audio::AudioRecorder::new());
    std::fs::create_dir_all(&settings.recording_dir)?;
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_name = format!("recording_{}.wav", timestamp);
    let output_path = settings.recording_dir.join(file_name);
    info!("Recording to: {}", output_path.display());
    recorder.start(
        output_path.clone(),
        settings.sample_rate,
        settings.channels,
        settings.audio_device,
        settings.silence_threshold,
    );
    signal::ctrl_c().await?;
    info!("Stopping recording...");
    recorder.stop();
    info!("Recording stopped.");
    Ok(())
}
