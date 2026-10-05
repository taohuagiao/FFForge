//! ffmpeg process management: spawn, progress parsing, throttled event
//! emission, cancellation.
//!
//! Key design constraints (see docs/ffmpeg-gui-tech-stack.md §4):
//! - Progress comes from stdout via `-progress pipe:1 -nostats`
//!   (`key=value` lines), never from stderr `frame=` lines.
//! - Progress events are throttled to one per 100 ms.
//! - Arguments are always passed as an array; never string-concatenated.
//! - Cancellation kills the held child handle immediately.
//! - On Windows the child is spawned with CREATE_NO_WINDOW.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

use crate::binaries::{sidecar_path, NoWindow};

/// Interval between `progress` events pushed to the frontend.
const PROGRESS_TICK: Duration = Duration::from_millis(100);
/// How many stderr tail lines to keep for error reporting.
const STDERR_TAIL_LEN: usize = 8;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    id: u32,
    out_time_ms: f64,
    percent: Option<f64>,
    fps: Option<f64>,
    speed: Option<f64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JobLogEvent<'a> {
    id: u32,
    stream: &'a str,
    line: &'a str,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JobErrorEvent {
    id: u32,
    message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JobFinishedEvent {
    id: u32,
    code: i32,
    cancelled: bool,
}

/// Latest progress snapshot, updated by the stdout reader task.
#[derive(Debug, Default, Clone)]
struct Snapshot {
    /// ffmpeg's `out_time_ms` field is actually microseconds (long-standing
    /// quirk); we convert to real milliseconds here.
    out_time_ms: Option<f64>,
    fps: Option<f64>,
    speed: Option<f64>,
    stderr_tail: Vec<String>,
}

#[derive(Default)]
pub struct Jobs {
    next_id: AtomicU32,
    inner: Mutex<HashMap<u32, Arc<JobHandle>>>,
}

struct JobHandle {
    child: Mutex<Option<Child>>,
    cancelled: AtomicBool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartJobArgs {
    pub input: String,
    pub output: String,
    pub duration_ms: Option<f64>,
    pub extra_args: Vec<String>,
}

pub fn start_job(
    app: &AppHandle,
    jobs: &Jobs,
    args: StartJobArgs,
) -> Result<u32, String> {
    // Arguments are always passed as an array. `-progress pipe:1 -nostats`
    // must be set so progress lands on stdout as `key=value` lines.
    let mut cmd = Command::new(sidecar_path("ffmpeg"));
    cmd.args([
        "-hide_banner",
        "-nostdin",
        "-y",
        "-progress",
        "pipe:1",
        "-nostats",
        "-i",
    ])
    .arg(&args.input)
    .args(&args.extra_args)
    .arg(&args.output)
    .no_window()
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .stdin(std::process::Stdio::null());

    let mut child = cmd.spawn().map_err(|e| format!("failed to spawn ffmpeg: {e}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let id = jobs.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let handle = Arc::new(JobHandle {
        child: Mutex::new(Some(child)),
        cancelled: AtomicBool::new(false),
    });
    jobs.inner
        .lock()
        .unwrap()
        .insert(id, handle.clone());

    let snapshot = Arc::new(Mutex::new(Snapshot::default()));

    // Reader tasks: parse stdout progress lines / forward stderr lines.
    if let Some(stdout) = stdout {
        let snapshot = snapshot.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                parse_progress_line(&line, &snapshot);
            }
        });
    }
    if let Some(stderr) = stderr {
        let snapshot = snapshot.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    let mut snap = snapshot.lock().unwrap();
                    snap.stderr_tail.push(line.clone());
                    if snap.stderr_tail.len() > STDERR_TAIL_LEN {
                        snap.stderr_tail.remove(0);
                    }
                    drop(snap);
                    let _ = app.emit(
                        "job-log",
                        JobLogEvent {
                            id,
                            stream: "stderr",
                            line: &line,
                        },
                    );
                }
            }
        });
    }

    // Monitor task: emit throttled progress events, apply cancellation,
    // reap the child, report completion.
    let app = app.clone();
    let duration_ms = args.duration_ms;
    tauri::async_runtime::spawn(async move {
        let jobs_state = app.state::<Jobs>();
        let mut ticker = tokio::time::interval(PROGRESS_TICK);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;

            // Cancellation: kill the held child handle.
            if handle.cancelled.load(Ordering::Relaxed) {
                if let Some(child) = handle.child.lock().unwrap().as_mut() {
                    let _ = child.start_kill();
                }
            }

            // Reap the child if it has exited.
            let status = {
                let mut guard = handle.child.lock().unwrap();
                match guard.as_mut().map(Child::try_wait) {
                    Some(Ok(Some(status))) => {
                        guard.take();
                        Some(Ok(status))
                    }
                    Some(Ok(None)) => None,
                    Some(Err(e)) => {
                        guard.take();
                        Some(Err(e))
                    }
                    None => break, // already reaped (e.g. by a previous iteration)
                }
            };

            // Throttled progress event (one per tick, not per line).
            {
                let snap = snapshot.lock().unwrap().clone();
                let out_time_ms = snap.out_time_ms.unwrap_or(0.0);
                let percent = duration_ms
                    .filter(|d| *d > 0.0)
                    .map(|d| Some((out_time_ms / d * 100.0).clamp(0.0, 100.0)))
                    .unwrap_or(None);
                let _ = app.emit(
                    "progress",
                    ProgressEvent {
                        id,
                        out_time_ms,
                        percent,
                        fps: snap.fps,
                        speed: snap.speed,
                    },
                );

                if let Some(result) = status {
                    let (code, poll_err) = match result {
                        Ok(status) => (status.code().unwrap_or(-1), None),
                        Err(e) => (-1, Some(format!("failed to poll ffmpeg process: {e}"))),
                    };
                    let cancelled = handle.cancelled.load(Ordering::Relaxed);
                    if let Some(msg) = poll_err {
                        let _ = app.emit("job-error", JobErrorEvent { id, message: msg });
                    } else if code != 0 && !cancelled {
                        let message = snapshot
                            .lock()
                            .unwrap()
                            .stderr_tail
                            .join("\n");
                        let _ = app.emit("job-error", JobErrorEvent { id, message });
                    }
                    let _ = app.emit(
                        "job-finished",
                        JobFinishedEvent { id, code, cancelled },
                    );
                    jobs_state.inner.lock().unwrap().remove(&id);
                    break;
                }
            }
        }
    });

    Ok(id)
}

pub fn cancel_job(jobs: &Jobs, id: u32) -> Result<(), String> {
    let handle = jobs.inner.lock().unwrap().get(&id).cloned();
    match handle {
        Some(handle) => {
            handle.cancelled.store(true, Ordering::Relaxed);
            Ok(())
        }
        None => Err(format!("job {id} not found")),
    }
}

/// Parse one `key=value` line from `ffmpeg -progress pipe:1`.
fn parse_progress_line(line: &str, snapshot: &Mutex<Snapshot>) {
    let Some((key, value)) = line.split_once('=') else {
        return;
    };
    let mut snap = snapshot.lock().unwrap();
    match key.trim() {
        // `out_time_ms` is microseconds despite the name; same for
        // `out_time_us`. Convert to milliseconds.
        "out_time_ms" | "out_time_us" => {
            snap.out_time_ms = value.trim().parse::<f64>().ok().map(|us| us / 1000.0);
        }
        "fps" => {
            snap.fps = value.trim().parse::<f64>().ok();
        }
        "speed" => {
            snap.speed = value.trim().trim_end_matches('x').parse::<f64>().ok();
        }
        _ => {}
    }
}
