mod launch;
use std::{collections::VecDeque, path::PathBuf, sync::Mutex};
#[cfg(target_os = "macos")]
use tauri::{Emitter, Manager};

struct OpenFiles(Mutex<VecDeque<PathBuf>>);

#[tauri::command]
fn launch_file_name(files: tauri::State<'_, OpenFiles>) -> Option<String> {
    files.0.lock().ok()?.front()?.file_name().map(|name| name.to_string_lossy().into_owned())
}

#[tauri::command]
fn read_launch_file(files: tauri::State<'_, OpenFiles>) -> Result<tauri::ipc::Response, String> {
    let path = files.0.lock().map_err(|error| error.to_string())?.pop_front().ok_or("No file to open")?;
    launch::read_launch_file(&path).map(tauri::ipc::Response::new)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = launch::launch_paths(std::env::args_os().skip(1));
    tauri::Builder::default()
        .manage(OpenFiles(Mutex::new(paths.into())))
        .invoke_handler(tauri::generate_handler![launch_file_name, read_launch_file])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .build(tauri::generate_context!())
        .expect("Unable to start OpenAEC Paint Studio")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                let state = _app.state::<OpenFiles>();
                if let Ok(mut paths) = state.0.lock() {
                    paths.extend(urls.into_iter().filter_map(|url| url.to_file_path().ok()).filter(|path| path.is_file()));
                }
                let _ = _app.emit("paint-open-files", ());
            }
        });
}
