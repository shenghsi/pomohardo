// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod input_blocker;
mod timer;

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

#[derive(Clone)]
struct AppState {
    timer: Arc<Mutex<timer::TimerEngine>>,
    config: Arc<Mutex<config::Config>>,
}

#[tauri::command]
async fn start_timer(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut timer = state.timer.lock().await;
    timer.start();
    Ok(())
}

#[tauri::command]
async fn pause_timer(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut timer = state.timer.lock().await;
    timer.pause();
    Ok(())
}

#[tauri::command]
async fn resume_timer(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut timer = state.timer.lock().await;
    timer.resume();
    Ok(())
}

#[tauri::command]
async fn skip_work(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut timer = state.timer.lock().await;
    timer.skip_work().map_err(|e| e.to_string())
}

#[tauri::command]
async fn request_emergency_skip(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let mut timer = state.timer.lock().await;
    let config = state.config.lock().await;
    timer.request_emergency_skip(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_timer_state(state: tauri::State<'_, AppState>) -> Result<timer::TimerState, String> {
    let timer = state.timer.lock().await;
    Ok(timer.get_state())
}

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<config::Config, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
async fn update_config(
    state: tauri::State<'_, AppState>,
    new_config: config::Config,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    *config = new_config.clone();
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_break_debt(state: tauri::State<'_, AppState>) -> Result<u32, String> {
    let timer = state.timer.lock().await;
    Ok(timer.get_break_debt())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let config = config::Config::load().unwrap_or_default();
            let timer = timer::TimerEngine::new(config.clone());

            let app_state = AppState {
                timer: Arc::new(Mutex::new(timer)),
                config: Arc::new(Mutex::new(config)),
            };

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_timer,
            pause_timer,
            resume_timer,
            skip_work,
            request_emergency_skip,
            get_timer_state,
            get_config,
            update_config,
            get_break_debt,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

