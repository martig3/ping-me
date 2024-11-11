use futures::future::try_join_all;
use screenshots::Screen;
use serde::Serialize;
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;
use tempfile::Builder;

use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tauri::State;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
#[cfg(debug_assertions)]
const INITIAL_URL: &str = "http://localhost:1420";
#[cfg(not(debug_assertions))]
const INITIAL_URL: &str = "tauri://localhost";
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
    phrases: Vec<String>,
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
                    match detect(&app, &phrases).await {
                        Ok(found_phrases) => {
                            if found_phrases.len() > 0 {
                                app.emit("notified", &found_phrases).unwrap();
                            }
                            println!("Work complete - Found: {:?}", found_phrases);
                            let secs = if found_phrases.len() > 0 { 2 * 60 } else { 5 };
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

async fn detect(app: &AppHandle, phrases: &Vec<String>) -> anyhow::Result<Vec<String>> {
    let temp_dir = Builder::new().prefix("screenshot_monitor").tempdir()?;
    let screens = Screen::all()?;
    let search_futures: Vec<_> = screens
        .iter()
        .map(|screen| {
            let temp_path = temp_dir
                .path()
                .join(format!("temp_screenshot_{}.png", screen.display_info.id));

            async move {
                let image = screen.capture()?;
                image.save(&temp_path).expect("Failed to save image");
                let output = app
                    .shell()
                    .sidecar("tesseract-ocr")
                    .unwrap()
                    .envs([("TESSDATA_PREFIX", "./binaries/tesseract-ocr/tessdata")])
                    .args([
                        temp_path.to_str().unwrap().to_string(),
                        "stdout".to_string(),
                    ])
                    .output()
                    .await?;
                let text = if output.status.success() {
                    String::from_utf8(output.stdout)?
                } else {
                    use anyhow::anyhow;
                    return Err(anyhow!(format!(
                        "tesseract stderr: {}",
                        String::from_utf8(output.stderr)?
                    )));
                };

                let found_phrases: Vec<String> = phrases
                    .iter()
                    .map(|p| p.to_string())
                    .filter(|p| text.contains(p))
                    .collect();
                if found_phrases.len() > 0 {
                    println!("Found matching phrase");
                    println!("Screen: {}", screen.display_info.id);
                    println!("Time: {}", chrono::Local::now());
                }

                if temp_path.exists() {
                    tokio::fs::remove_file(&temp_path).await?;
                }

                Ok::<Vec<String>, anyhow::Error>(found_phrases)
            }
        })
        .collect();

    let results = try_join_all(search_futures).await?;
    let found_phrases = results
        .into_iter()
        .flat_map(|f| f)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    Ok(found_phrases)
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
async fn is_notifying(state: State<'_, Arc<NotifyState>>) -> Result<NotifyStateResponse, String> {
    Ok(NotifyStateResponse {
        is_running: *state.is_running.lock().await,
    })
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
        .setup(|app| {
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
                let main_webview = app.get_webview_window("main").unwrap();
                app.deep_link().on_open_url(move |_event| {
                    main_webview
                        .eval(&format!("window.location.href = '{}'", INITIAL_URL))
                        .unwrap();
                });
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(NotifyState::new()))
        .invoke_handler(tauri::generate_handler![
            start_notifying,
            stop_notifying,
            is_notifying
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
