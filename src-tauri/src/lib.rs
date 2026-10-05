mod binaries;
mod ffmpeg;
mod probe;

use tauri::{AppHandle, State};

use ffmpeg::{Jobs, StartJobArgs};

#[tauri::command]
async fn probe(path: String) -> Result<probe::ProbeResult, String> {
    probe::probe(&path).await
}

#[tauri::command]
fn start_job(
    app: AppHandle,
    state: State<'_, Jobs>,
    args: StartJobArgs,
) -> Result<u32, String> {
    ffmpeg::start_job(&app, state.inner(), args)
}

#[tauri::command]
fn cancel_job(state: State<'_, Jobs>, id: u32) -> Result<(), String> {
    ffmpeg::cancel_job(state.inner(), id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Jobs::default())
        .invoke_handler(tauri::generate_handler![probe, start_job, cancel_job])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
