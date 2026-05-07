// Coder's Bible Desktop — Tauri v2 Entry Point
// Sovereign knowledge engine. Zero AI. Zero network.

use std::sync::Mutex;
use tauri::{menu::*, tray::*, Manager, State, WebviewUrl, WebviewWindowBuilder};

mod bible_engine;
use bible_engine::{AnalysisResult, BibleEngine, SearchResult, StatsResult};

struct AppState {
    engine: Mutex<BibleEngine>,
}

// ─── Tauri Commands ─────────────────────────────────────────

#[tauri::command]
fn search(state: State<'_, AppState>, query: String, limit: Option<u32>) -> Result<SearchResult, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(20).min(50) as usize;
    Ok(engine.search(&query, limit))
}

#[tauri::command]
fn analyze(state: State<'_, AppState>, snippet: String) -> Result<AnalysisResult, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    if snippet.len() > 10000 {
        return Err("Snippet too large (max 10,000 chars)".into());
    }
    Ok(engine.analyze(&snippet))
}

#[tauri::command]
fn stats(state: State<'_, AppState>) -> Result<StatsResult, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_stats())
}

#[tauri::command]
fn health(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let count = engine.total_fragments();
    Ok(serde_json::json!({
        "status": "ok",
        "engine": "bible_engine_rust",
        "fragments": count,
    }))
}

// ─── App Setup ──────────────────────────────────────────────

fn setup_db(app: &tauri::App) -> Result<BibleEngine, Box<dyn std::error::Error>> {
    let app_data = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data)?;
    let db_path = app_data.join("coders_bible.db");

    // On first run, copy bundled DB to app data dir
    if !db_path.exists() {
        let bundled = app.path().resolve(
            "coders_bible.db",
            tauri::path::BaseDirectory::Resource,
        )?;
        std::fs::copy(&bundled, &db_path)?;
        log::info!("Copied bundled DB to {:?}", db_path);
    }

    Ok(BibleEngine::new(db_path.to_string_lossy().to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let engine = setup_db(app).map_err(|e| e.to_string())?;
            app.manage(AppState { engine: Mutex::new(engine) });

            // Tray icon
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Coder's Bible", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Global shortcut: Ctrl+Shift+Space opens mini search window
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                let shortcut = Shortcut::new(Some(tauri::Modifiers::CONTROL | tauri::Modifiers::SHIFT), tauri::Key::Space);
                app.global_shortcut().register(shortcut, move |app| {
                    if let Some(window) = app.get_webview_window("mini") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    } else if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                })?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![search, analyze, stats, health])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    env_logger::init();
    run();
}
