use std::path::PathBuf;
use tracing::error;
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
        .output()
        .unwrap();
    if !status.status.success() {
        let err = String::from_utf8_lossy(&status.stderr);
        error!("ffmpeg failed: {}", err);
        return Err(std::io::Error::other("ffmpeg failed"));
    }
    Ok(())
}
