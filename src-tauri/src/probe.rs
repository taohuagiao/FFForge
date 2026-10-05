//! ffprobe metadata probing.
//!
//! Runs `ffprobe -print_format json -show_format -show_streams <path>` and
//! extracts the fields required by the IPC contract (`probe` command).

use serde::Serialize;
use serde_json::Value;

use crate::binaries::{sidecar_path, NoWindow};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    /// Total duration in milliseconds (`format.duration` seconds * 1000).
    pub duration_ms: f64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
}

pub async fn probe(path: &str) -> Result<ProbeResult, String> {
    let output = tokio::process::Command::new(sidecar_path("ffprobe"))
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .no_window()
        .output()
        .await
        .map_err(|e| format!("failed to launch ffprobe: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("ffprobe exited with {}", output.status)
        } else {
            stderr
        });
    }

    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("failed to parse ffprobe output: {e}"))?;

    let duration_s = json
        .pointer("/format/duration")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let mut result = ProbeResult {
        duration_ms: duration_s * 1000.0,
        width: None,
        height: None,
        vcodec: None,
        acodec: None,
    };

    if let Some(streams) = json.get("streams").and_then(Value::as_array) {
        for stream in streams {
            let codec_type = stream.get("codec_type").and_then(Value::as_str);
            let codec_name = stream
                .get("codec_name")
                .and_then(Value::as_str)
                .map(|s| s.to_string());
            match codec_type {
                Some("video") if result.vcodec.is_none() => {
                    result.vcodec = codec_name;
                    result.width = stream.get("width").and_then(Value::as_u64).map(|v| v as u32);
                    result.height = stream
                        .get("height")
                        .and_then(Value::as_u64)
                        .map(|v| v as u32);
                }
                Some("audio") if result.acodec.is_none() => {
                    result.acodec = codec_name;
                }
                _ => {}
            }
        }
    }

    Ok(result)
}
