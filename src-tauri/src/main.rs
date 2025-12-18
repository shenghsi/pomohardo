// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod input_blocker;
mod timer;
mod tray;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder, RunEvent, WindowEvent};

#[derive(Clone)]
struct AppState {
    timer: Arc<Mutex<timer::TimerEngine>>,
    config: Arc<Mutex<config::Config>>,
    input_blocker: Arc<Mutex<input_blocker::InputBlocker>>,
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
    mut new_config: config::Config,
) -> Result<(), String> {
    // Avoid deadlocks: never hold config+timer locks at the same time (other commands lock timer->config).

    let (limit_locked, old_emergency_limit) = {
        let timer = state.timer.lock().await;
        let locked = timer.get_state().emergency_limit_locked;
        drop(timer);

        let config = state.config.lock().await;
        (locked, config.emergency_skips_per_day)
    };

    // If the daily limit is locked (after the first work session ends), ignore any attempt to modify it.
    if limit_locked {
        new_config.emergency_skips_per_day = old_emergency_limit;
    }

    {
        let mut config = state.config.lock().await;
        *config = new_config.clone();
        config.save().map_err(|e| e.to_string())?;
    }

    // Keep the timer engine in sync so durations + break-debt calculations use latest config.
    let mut timer = state.timer.lock().await;
    timer.update_config(new_config);
    Ok(())
}

#[tauri::command]
async fn get_break_debt(state: tauri::State<'_, AppState>) -> Result<u32, String> {
    let timer = state.timer.lock().await;
    Ok(timer.get_break_debt())
}

#[tauri::command]
async fn activate_input_blocking(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut blocker = state.input_blocker.lock().await;
    blocker.activate()
}

#[tauri::command]
async fn deactivate_input_blocking(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut blocker = state.input_blocker.lock().await;
    blocker.deactivate()
}

#[tauri::command]
async fn emergency_chord_pressed(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let mut blocker = state.input_blocker.lock().await;
    #[cfg(target_os = "linux")]
    {
        return blocker.emergency_chord_pressed();
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(false)
    }
}

#[tauri::command]
async fn complete_break(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut timer = state.timer.lock().await;
    timer.complete_break();
    Ok(())
}

#[tauri::command]
async fn show_breakshield(app: tauri::AppHandle, url: String) -> Result<(), String> {
    // In dev mode, the main window loads from an external devUrl (http://localhost:5173),
    // so a breakshield window must also load from the same external URL.
    // In production, `url` will typically be an `app://` URL and will still parse fine.

    if let Some(w) = app.get_webview_window("breakshield") {
        let _ = w.close();
    }

    let webview_url = match tauri::Url::parse(&url) {
        Ok(u) => WebviewUrl::External(u),
        Err(_) => WebviewUrl::App("index.html#breakshield".into()),
    };

    // Fullscreen overlay window. We use a transparent window + a dark scrim in CSS.
    let w = WebviewWindowBuilder::new(&app, "breakshield", webview_url)
        .title("Pomohardo Break")
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .fullscreen(true)
        .build()
        .map_err(|e| e.to_string())?;

    // Some window managers ignore fullscreen-at-create for borderless/transparent windows.
    // Force it after creation and also hard-set size/position to the current monitor bounds.
    let _ = w.show();
    let _ = w.set_focus();
    let _ = w.set_fullscreen(true);
    let _ = w.set_always_on_top(true);
    if let Ok(Some(mon)) = w.current_monitor() {
        let pos = *mon.position();
        let size = *mon.size();
        let _ = w.set_position(pos);
        let _ = w.set_size(size);
    }

    Ok(())
}

#[tauri::command]
async fn hide_breakshield(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("breakshield") {
        let _ = w.close();
    }
    Ok(())
}

#[tauri::command]
async fn show_settings(app: tauri::AppHandle, url: String) -> Result<(), String> {
    // If settings window already exists, just focus it
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_focus();
        return Ok(());
    }

    let webview_url = match tauri::Url::parse(&url) {
        Ok(u) => WebviewUrl::External(u),
        Err(_) => WebviewUrl::App("settings.html".into()),
    };

    // Settings window - sized to fit content without scrolling
    // 9 settings * ~56px each + header + padding = ~620px height
    let w = WebviewWindowBuilder::new(&app, "settings", webview_url)
        .title("Preferences")
        .inner_size(450.0, 600.0)
        .resizable(false)
        .decorations(true)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

    let _ = w.show();
    let _ = w.set_focus();

    Ok(())
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
                input_blocker: Arc::new(Mutex::new(input_blocker::InputBlocker::new())),
            };

            // Create system tray icon
            let tray = tray::create_tray(app.handle())
                .expect("Failed to create system tray");
            
            let app_handle = app.handle().clone();
            let timer_clone = app_state.timer.clone();
            let tray_clone = tray.clone();

            // Background task to check timer state and trigger transitions
            std::thread::spawn(move || {
                let mut last_phase = timer::Phase::Work;
                let rt = tokio::runtime::Runtime::new().unwrap();

                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    
                    // Use blocking lock since we're in a regular thread
                    let mut timer = futures::executor::block_on(timer_clone.lock());
                    
                    // Check if phase transition is needed
                    let transitioned = timer.check_and_transition();
                    
                    let current_state = timer.get_state();
                    let current_phase = current_state.phase;
                    drop(timer);

                    // Update tray icon with current progress
                    rt.block_on(async {
                        if let Err(e) = tray::update_tray_icon(&tray_clone, &timer_clone).await {
                            eprintln!("Failed to update tray icon: {}", e);
                        }
                    });

                    // Emit event if phase changed
                    if transitioned || last_phase != current_phase {
                        last_phase = current_phase;
                        
                        let phase_name = match current_phase {
                            timer::Phase::Work => "work",
                            timer::Phase::Break => "break",
                            timer::Phase::LongBreak => "long_break",
                        };

                        app_handle.emit("phase-changed", phase_name).ok();
                        
                        // Show notification for break start
                        if matches!(current_phase, timer::Phase::Break | timer::Phase::LongBreak) {
                            let break_type = if matches!(current_phase, timer::Phase::LongBreak) {
                                "Long break"
                            } else {
                                "Break"
                            };
                            
                            app_handle
                                .emit("break-started", format!("{} time! Take a break.", break_type))
                                .ok();
                        }
                    }
                }
            });

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
            activate_input_blocking,
            deactivate_input_blocking,
            emergency_chord_pressed,
            show_breakshield,
            hide_breakshield,
            show_settings,
            complete_break,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::WindowEvent {
                    label,
                    event: WindowEvent::CloseRequested { api, .. },
                    ..
                } => {
                    // For main window, hide instead of close
                    if label == "main" {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                }
                _ => {}
            }
        });
}

