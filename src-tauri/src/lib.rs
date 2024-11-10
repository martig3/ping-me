use futures::future::try_join_all;
use screenshots::Screen;
use serde::Serialize;
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tempfile::Builder;
use tokio::process::Command;

use std::env;
use std::sync::Arc;
use tauri::State;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

struct NotifyState {
    cancel_tx: broadcast::Sender<()>,
    is_running: Mutex<bool>,
}

impl NotifyState {
    fn new() -> Self {
        let (cancel_tx, _) = broadcast::channel(1);
        Self {
            cancel_tx,
            is_running: Mutex::new(false),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NotifyStateResponse {
    is_running: bool,
}

#[tauri::command]
async fn start_notifying(
    app: AppHandle,
    state: State<'_, Arc<NotifyState>>,
) -> Result<NotifyStateResponse, String> {
    let mut is_running = state.is_running.lock().await;

    if *is_running {
        return Ok(NotifyStateResponse { is_running: true });
    }

    *is_running = true;
    let mut cancel_rx = state.cancel_tx.subscribe();

    let state_clone = state.inner().clone();

    tokio::spawn(async move {
        loop {
            select! {
                _ = async {
                    println!("Working...");
                    match detect().await {
                        Ok(found) => {
                            if found {
                                app.emit("notified", ()).unwrap();
                            }
                            println!("Work complete - Found: {}", found);
                            let secs = if found { 2 * 60 } else { 5 };
                            println!("sleeping for {}s", &secs);
                            sleep(Duration::from_secs(secs)).await;
                        },
                        Err(err) => println!("Worker error: {:?}", err),
                    };
                } => {}

                Ok(()) = cancel_rx.recv() => {
                    println!("Loop cancelled during execution!");
                    let mut is_running = state_clone.is_running.lock().await;
                    *is_running = false;
                    break;
                }
            }
        }
        println!("Loop stopped cleanly");
    });

    Ok(NotifyStateResponse { is_running: true })
}

async fn detect() -> anyhow::Result<bool> {
    let temp_dir = Builder::new().prefix("screenshot_monitor").tempdir()?;
    let search_phrase = "Enter Dungeon";
    let screens = Screen::all()?;
    let temp_dir_path = temp_dir.path();
    let search_futures: Vec<_> = screens
        .iter()
        .map(|screen| {
            let temp_path = temp_dir
                .path()
                .join(format!("temp_screenshot_{}.png", screen.display_info.id));
            let search_phrase = search_phrase.to_string();

            async move {
                let image = screen.capture()?;
                image.save(&temp_path).expect("Failed to save image");
                let mut cmd = Command::new("cmd");
                cmd.current_dir(&temp_dir_path);
                cmd.args([
                    "/C".to_string(),
                    "C:\\Program Files\\Tesseract-OCR\\tesseract.exe".to_string(),
                    format!("temp_screenshot_{}.png", screen.display_info.id),
                    "stdout".to_string(),
                ]);
                let output = cmd.output().await?;
                let text = if output.status.success() {
                    String::from_utf8(output.stdout)?
                } else {
                    use anyhow::anyhow;
                    return Err(anyhow!(format!(
                        "tesseract stderr: {}",
                        String::from_utf8(output.stderr)?
                    )));
                };

                let found = text.contains(&search_phrase);
                if found {
                    println!("Found matching phrase");
                    println!("Screen: {}", screen.display_info.id);
                    println!("Time: {}", chrono::Local::now());
                }

                if temp_path.exists() {
                    tokio::fs::remove_file(&temp_path).await?;
                }

                Ok::<bool, anyhow::Error>(found)
            }
        })
        .collect();

    let results = try_join_all(search_futures).await?;
    let found = results
        .into_iter()
        .reduce(|acc, x| acc || x)
        .unwrap_or(false);

    Ok(found)
}

#[tauri::command]
async fn stop_notifying(state: State<'_, Arc<NotifyState>>) -> Result<NotifyStateResponse, String> {
    let is_running = *state.is_running.lock().await;

    if !is_running {
        return Ok(NotifyStateResponse { is_running: false });
    }

    let _ = state.cancel_tx.send(());
    Ok(NotifyStateResponse { is_running: false })
}

#[tauri::command]
async fn is_notifying(state: State<'_, Arc<NotifyState>>) -> Result<bool, String> {
    Ok(*state.is_running.lock().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }
    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            }
            Ok(())
        })
        .manage(Arc::new(NotifyState::new()))
        .invoke_handler(tauri::generate_handler![
            start_notifying,
            stop_notifying,
            is_notifying
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
