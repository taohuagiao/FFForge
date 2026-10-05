use std::path::PathBuf;

/// Extension trait: spawn child processes without a console window on Windows
/// (`CREATE_NO_WINDOW` = 0x08000000). No-op elsewhere.
pub trait NoWindow {
    fn no_window(&mut self) -> &mut Self;
}

#[cfg(windows)]
impl NoWindow for tokio::process::Command {
    fn no_window(&mut self) -> &mut Self {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        self.creation_flags(CREATE_NO_WINDOW)
    }
}

#[cfg(not(windows))]
impl NoWindow for tokio::process::Command {
    fn no_window(&mut self) -> &mut Self {
        self
    }
}

/// Resolve the path of a bundled sidecar binary (ffmpeg / ffprobe).
///
/// Lookup order:
/// 1. Next to the app executable (release/bundled layout; Tauri's
///    `bundle.externalBin` copies binaries there with the target triple
///    suffix stripped).
/// 2. `src-tauri/binaries/<name>-<triple>[.exe]` (dev layout; compile-time
///    manifest dir works because `tauri dev` runs on the same checkout).
/// 3. Bare name as a fallback to PATH.
pub fn sidecar_path(name: &str) -> PathBuf {
    let exe_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };

    // 1. Bundled layout: binary sits next to the app executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join(&exe_name);
            if p.is_file() {
                return p;
            }
        }
    }

    // 2. Dev layout: src-tauri/binaries/<name>-<triple>[.exe]
    let triple = if cfg!(windows) {
        "x86_64-pc-windows-msvc".to_string()
    } else if cfg!(target_os = "macos") {
        format!("{}-apple-darwin", std::env::consts::ARCH)
    } else {
        format!("{}-unknown-linux-gnu", std::env::consts::ARCH)
    };
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(if cfg!(windows) {
            format!("{name}-{triple}.exe")
        } else {
            format!("{name}-{triple}")
        });
    if dev.is_file() {
        return dev;
    }

    // 3. Fallback: rely on PATH.
    PathBuf::from(exe_name)
}
