// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod about;
mod input_blocker;
mod timer;
mod tray;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tauri::{Emitter, Manager, Listener, WebviewUrl, WebviewWindowBuilder, RunEvent, WindowEvent, AppHandle};
use tauri_plugin_autostart::MacosLauncher;

#[derive(Clone)]
struct AppState {
    timer: Arc<Mutex<timer::TimerEngine>>,
    config: Arc<Mutex<config::Config>>,
    input_blocker: Arc<Mutex<input_blocker::InputBlocker>>,
}

#[derive(serde::Serialize)]
struct AboutInfo {
    version: String,
    icon_base64: String,
}

#[tauri::command]
async fn get_about_info(app: AppHandle) -> Result<AboutInfo, String> {
    use base64::{engine::general_purpose, Engine as _};
    
    let version = app.package_info().version.to_string();
    let icon_data = include_bytes!("../icons/icon.svg");
    let icon_base64 = general_purpose::STANDARD.encode(icon_data);
    let icon_src = format!("data:image/svg+xml;base64,{}", icon_base64);

    Ok(AboutInfo {
        version,
        icon_base64: icon_src,
    })
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
    app: AppHandle,
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
        
        // Handle auto-start changes
        if config.auto_start != new_config.auto_start {
            use tauri_plugin_autostart::ManagerExt;
            if new_config.auto_start {
                let _ = app.autolaunch().enable();
            } else {
                let _ = app.autolaunch().disable();
            }
        }

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
    blocker.emergency_chord_pressed()
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
        .resizable(false)
        .always_on_top(true)
        .fullscreen(true)
        .build()
        .map_err(|e: tauri::Error| e.to_string())?;

    // Some window managers ignore fullscreen-at-create for borderless/transparent windows.
    // Force it after creation and also hard-set size/position to the current monitor bounds.
    let _ = w.show();
    let _ = w.set_focus();
    let _ = w.set_fullscreen(true);
    let _ = w.set_always_on_top(true);
    if let Ok(Some(mon)) = w.current_monitor() {
        let pos = *mon.position();
        let size = *mon.size();
        let _ = w.set_position(tauri::Position::Physical(pos));
        let _ = w.set_size(tauri::Size::Physical(size));
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

pub const SETTINGS_WINDOW_WIDTH: f64 = 450.0;
pub const SETTINGS_WINDOW_HEIGHT: f64 = 700.0;

pub fn open_settings_window(app: &tauri::AppHandle) -> Result<(), String> {
    // If settings window already exists, close it and recreate with new size
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }

    // Settings window - sized to fit content without scrolling
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Preferences")
        .inner_size(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(true)
        .center()
        .build()
        .map(|w| {
            let _ = w.show();
            let _ = w.set_focus();
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_settings(app: tauri::AppHandle, _url: String) -> Result<(), String> {
    open_settings_window(&app)
}

/// Linux-specific single instance lock using filesystem-based locking.
///
/// This function works alongside `tauri-plugin-single-instance` to provide robust
/// single-instance enforcement on Linux. While the Tauri plugin handles the primary
/// single-instance logic, this filesystem lock addresses a specific race condition
/// that can occur during rapid application restarts.
///
/// # Why Both Mechanisms Are Needed
///
/// The `tauri-plugin-single-instance` plugin uses inter-process communication (IPC)
/// to detect running instances. However, there's a brief window during application
/// shutdown where:
/// 1. The first instance has released its IPC resources
/// 2. But hasn't fully terminated yet
/// 3. A second instance can start and pass the plugin's check
/// 4. Both instances end up running simultaneously
///
/// This filesystem lock prevents that race by:
/// - Creating an atomic lock directory in `/tmp/pomohardo_lock`
/// - Writing the process PID to a file within that directory
/// - Checking if the PID corresponds to a running process via `/proc/{pid}`
/// - Cleaning up stale locks from crashed instances
///
/// # Lock Lifecycle
///
/// - **Acquisition**: Atomically creates lock directory, writes PID
/// - **Validation**: Checks `/proc/{pid}` to verify process is alive
/// - **Cleanup**: Removes stale locks from dead processes
/// - **Release**: Lock is automatically released when process exits (OS cleans up `/tmp`)
///
/// # Returns
///
/// - `true` if this is the first/only instance (lock acquired)
/// - `false` if another instance is already running (lock held by valid process)
#[cfg(target_os = "linux")]
fn acquire_instance_lock() -> bool {
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    let lock_dir = std::env::temp_dir().join("pomohardo_lock");
    let pid_file = lock_dir.join("pid");

    // Try to create the directory - this is atomic
    match fs::create_dir(&lock_dir) {
        Ok(_) => {
            // We succeeded, so we are the first instance.
            let pid = std::process::id();
            if let Err(e) = fs::write(&pid_file, pid.to_string()) {
                eprintln!("Failed to write PID to lock file: {}", e);
                // If we can't write, we should probably clean up and allow others (or fail).
                // But failure to write to tmp is critical.
            }
            true
        }
        Err(_) => {
            // Directory exists.
            // 1. Check if the lock is held by a valid process.
            // we loop a few times to handle the race where P1 created dir but hasn't written PID yet.
            for _ in 0..10 {
                if let Ok(content) = fs::read_to_string(&pid_file) {
                    if let Ok(pid) = content.trim().parse::<i32>() {
                        if Path::new(&format!("/proc/{}", pid)).exists() {
                            eprintln!("Instance already running at PID {}", pid);
                            return false; // Valid lock found, we are duplicate.
                        } else {
                            // PID file exists but process is dead. Stale lock.
                            break; 
                        }
                    }
                }
                // PID file missing or unreadable (maybe P1 is writing). Wait and retry.
                thread::sleep(Duration::from_millis(50));
            }
            
            // If we are here, either the process is dead (stale) or we timed out waiting for PID file.
            // We interpret this as "No valid instance running".
            // Remove stale lock.
            let _ = fs::remove_dir_all(&lock_dir);
            
            // Retry acquisition once
            if fs::create_dir(&lock_dir).is_ok() {
                let pid = std::process::id();
                let _ = fs::write(&pid_file, pid.to_string());
                return true;
            }
            // If retry failed, someone else grabbed it in the microsecond between remove and create.
            // We assume they are valid.
            false
        }
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    if !acquire_instance_lock() {
        std::process::exit(0);
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance tries to start, bring the existing window to focus
            if let Some(window) = app.get_webview_window("main") {
                // Ensure window is visible, unminimized, and focused
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // Prevent double initialization
            use std::sync::atomic::{AtomicBool, Ordering};
            static INITIALIZED: AtomicBool = AtomicBool::new(false);
            if INITIALIZED.swap(true, Ordering::SeqCst) {
                 return Ok(());
            }

            // macOS: Create custom app menu with controlled Quit behavior
            #[cfg(target_os = "macos")]
            {
                use tauri::menu::{Menu, Submenu, MenuItem, PredefinedMenuItem};
                
                let app_handle = app.handle();
                
                // Create custom Quit menu item
                let quit_item = MenuItem::with_id(app_handle, "quit", "Quit Pomohardo", true, Some("Cmd+Q"))?;
                
                // Create app submenu
                let app_submenu = Submenu::with_items(
                    app_handle,
                    "Pomohardo",
                    true,
                    &[
                        &PredefinedMenuItem::about(app_handle, Some("About Pomohardo"), None)?,
                        &PredefinedMenuItem::separator(app_handle)?,
                        &PredefinedMenuItem::services(app_handle, None)?,
                        &PredefinedMenuItem::separator(app_handle)?,
                        &PredefinedMenuItem::hide(app_handle, None)?,
                        &PredefinedMenuItem::hide_others(app_handle, None)?,
                        &PredefinedMenuItem::show_all(app_handle, None)?,
                        &PredefinedMenuItem::separator(app_handle)?,
                        &quit_item,
                    ],
                )?;
                
                // Create Edit submenu for standard text editing
                let edit_submenu = Submenu::with_items(
                    app_handle,
                    "Edit",
                    true,
                    &[
                        &PredefinedMenuItem::undo(app_handle, None)?,
                        &PredefinedMenuItem::redo(app_handle, None)?,
                        &PredefinedMenuItem::separator(app_handle)?,
                        &PredefinedMenuItem::cut(app_handle, None)?,
                        &PredefinedMenuItem::copy(app_handle, None)?,
                        &PredefinedMenuItem::paste(app_handle, None)?,
                        &PredefinedMenuItem::select_all(app_handle, None)?,
                    ],
                )?;
                
                // Create Window submenu
                let window_submenu = Submenu::with_items(
                    app_handle,
                    "Window",
                    true,
                    &[
                        &PredefinedMenuItem::minimize(app_handle, None)?,
                        &PredefinedMenuItem::maximize(app_handle, None)?,
                        &PredefinedMenuItem::close_window(app_handle, None)?,
                    ],
                )?;
                
                // Build the menu
                let menu = Menu::with_items(app_handle, &[&app_submenu, &edit_submenu, &window_submenu])?;
                
                // Set the menu
                app.set_menu(menu)?;
                
                // Handle custom quit menu item
                app.on_menu_event(move |app, event| {
                    if event.id().as_ref() == "quit" {
                        // Check if breakshield is active
                        if app.get_webview_window("breakshield").is_some() {
                            // Refocus breakshield
                            if let Some(breakshield) = app.get_webview_window("breakshield") {
                                let _ = breakshield.set_always_on_top(true);
                                let _ = breakshield.set_focus();
                            }
                        } else {
                            // Allow quit when not in break
                            app.exit(0);
                        }
                    }
                });
            }

            // Check command line args for --minimized
            let args: Vec<String> = std::env::args().collect();
            let start_minimized = args.contains(&"--minimized".to_string());

            // On autostart (--minimized), add a small delay to let the system settle
            // This helps prevent duplicate tray icons on Linux after login
            #[cfg(target_os = "linux")]
            if start_minimized {
                std::thread::sleep(Duration::from_millis(500));
            }

            if !start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            let config = config::Config::load().unwrap_or_default();
            let timer = timer::TimerEngine::new(config.clone());

            let app_state = AppState {
                timer: Arc::new(Mutex::new(timer)),
                config: Arc::new(Mutex::new(config)),
                input_blocker: Arc::new(Mutex::new(input_blocker::InputBlocker::new())),
            };

            // Create system tray icon
            let (tray, pause_resume_item) = tray::create_tray(app.handle())
                .expect("Failed to create system tray");
            
            let app_handle = app.handle().clone();
            let timer_clone = app_state.timer.clone();
            let tray_clone = tray.clone();
            let pause_resume_item_clone = pause_resume_item.clone();
            
            // Listen for tray update requests (e.g. from menu clicks)
            let tray_for_event = tray.clone();
            let item_for_event = pause_resume_item.clone();
            let timer_for_event = app_state.timer.clone();
            app.listen("refresh-tray-icon", move |_| {
                let t = tray_for_event.clone();
                let i = item_for_event.clone();
                let tm = timer_for_event.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = tray::update_tray_icon(&t, &tm, &i).await {
                        eprintln!("Failed to refresh tray icon: {}", e);
                    }
                });
            });

            // macOS: Aggressive focus enforcement thread to prevent escape (Cmd+Tab, Force Quit dialog, etc.)
            #[cfg(target_os = "macos")]
            {
                let app_handle_focus = app.handle().clone();
                let input_blocker_for_focus = app_state.input_blocker.clone();
                std::thread::spawn(move || {
                    loop {
                        // Very aggressive: check every 25ms to fight Force Quit dialog and Cmd+Tab
                        std::thread::sleep(Duration::from_millis(25));
                        
                        if let Some(breakshield) = app_handle_focus.get_webview_window("breakshield") {
                            // Check if input blocking is active - if not, don't enforce focus
                            // This allows user interaction when break time is up
                            let blocker_active = {
                                let blocker = futures::executor::block_on(input_blocker_for_focus.lock());
                                blocker.is_active()
                            };
                            
                            if !blocker_active {
                                // Input blocking is deactivated (break time is up), don't enforce focus
                                continue;
                            }
                            
                            // Always try to keep breakshield on top and focused during active break
                            // This fights against Force Quit dialog and other system windows
                            let _ = breakshield.set_always_on_top(true);
                            
                            // Always try to refocus - don't even check if focused first
                            // This is more aggressive and helps steal focus back from Force Quit dialog
                            let _ = breakshield.set_focus();
                        }
                    }
                });
            }

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
                        if let Err(e) = tray::update_tray_icon(&tray_clone, &timer_clone, &pause_resume_item_clone).await {
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
            show_settings,
            complete_break,
            get_about_info,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match &event {
                #[cfg(target_os = "macos")]
                RunEvent::Ready => {
                    // macOS menu is ready
                }
                RunEvent::ExitRequested { api, .. } => {
                    // Prevent quit when breakshield is active (break enforcement)
                    if app_handle.get_webview_window("breakshield").is_some() {
                        api.prevent_exit();
                        // Refocus the breakshield window
                        if let Some(breakshield) = app_handle.get_webview_window("breakshield") {
                            let _ = breakshield.set_always_on_top(true);
                            let _ = breakshield.set_focus();
                        }
                    }
                }
                RunEvent::WindowEvent {
                    label,
                    event: WindowEvent::CloseRequested { api, .. },
                    ..
                } => {
                    // If this IS the breakshield window being closed, allow it
                    // (This happens when break time is up and user interacts)
                    if label == "breakshield" {
                        return;
                    }
                    
                    let breakshield_exists = app_handle.get_webview_window("breakshield").is_some();
                    
                    // During break (breakshield exists), prevent closing other windows
                    if breakshield_exists {
                        api.prevent_close();
                        // Refocus the breakshield window
                        if let Some(breakshield) = app_handle.get_webview_window("breakshield") {
                            let _ = breakshield.set_always_on_top(true);
                            let _ = breakshield.set_focus();
                        }
                        return;
                    }
                    
                    // For main window, hide instead of close
                    if label == "main" {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                }
                RunEvent::WindowEvent {
                    label,
                    event: WindowEvent::Destroyed,
                    ..
                } => {
                    // Window destroyed - no action needed
                    let _ = label; // Suppress unused warning
                }
                RunEvent::Exit => {
                    // Clean up tray icon on exit to prevent ghost icons on Linux
                    if let Some(tray) = app_handle.tray_by_id("pomohardo-tray") {
                        let _ = tray.set_visible(false);
                    }
                    
                    // Clean up the instance lock on Linux
                    #[cfg(target_os = "linux")]
                    {
                        let lock_dir = std::env::temp_dir().join("pomohardo_lock");
                        let _ = std::fs::remove_dir_all(&lock_dir);
                    }
                }
                _ => {}
            }
        });
}

