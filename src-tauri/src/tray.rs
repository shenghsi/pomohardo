use image::{codecs::png::PngEncoder, ImageEncoder, ImageBuffer, Rgba, RgbaImage};
use std::f32::consts::PI;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Emitter,
};
use tokio::sync::Mutex;

use crate::timer::{Phase, TimerEngine, TimerStatus};

const ICON_SIZE: u32 = 32;
const RING_WIDTH: f32 = 4.0;
const CENTER: f32 = (ICON_SIZE as f32) / 2.0;
const RADIUS: f32 = CENTER - RING_WIDTH / 2.0 - 1.0;

// Tomato/pomodoro color for work phase
const WORK_COLOR: Rgba<u8> = Rgba([231, 76, 60, 255]); // #E74C3C
// Gray for the track (matches work session background ring)
const TRACK_COLOR: Rgba<u8> = Rgba([138, 138, 138, 150]); // #8a8a8a
// Green for break phase
const BREAK_COLOR: Rgba<u8> = Rgba([46, 204, 113, 255]); // #2ECC71
// White for play symbol
const PLAY_SYMBOL_COLOR: Rgba<u8> = Rgba([255, 255, 255, 255]); // #FFFFFF

/// Draw a play triangle symbol in the center
fn draw_play_symbol(img: &mut RgbaImage, color: Rgba<u8>) {
    let play_size = 13.0; // Size of the play triangle
    let play_center_x = CENTER;
    let play_center_y = CENTER;
    
    // Play triangle: three points forming a right-pointing triangle
    // Tip (right), Top-Left, Bottom-Left
    let tip_x = play_center_x + play_size * 0.58;
    let tip_y = play_center_y;
    
    let base_x = play_center_x - play_size * 0.29;
    let top_y = play_center_y - play_size * 0.5;
    let bottom_y = play_center_y + play_size * 0.5;
    
    // Draw triangle using barycentric coordinates
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let px = x as f32;
            let py = y as f32;
            
            // Barycentric coordinates
            // Triangle vertices: A(base_x, top_y), B(base_x, bottom_y), C(tip_x, tip_y)
            
            // Vectors from A
            let v0x = 0.0; // B.x - A.x
            let v0y = bottom_y - top_y; // B.y - A.y
            
            let v1x = tip_x - base_x; // C.x - A.x
            let v1y = tip_y - top_y; // C.y - A.y
            
            let v2x = px - base_x; // P.x - A.x
            let v2y = py - top_y; // P.y - A.y
            
            let dot00 = v0x * v0x + v0y * v0y;
            let dot01 = v0x * v1x + v0y * v1y;
            let dot02 = v0x * v2x + v0y * v2y;
            let dot11 = v1x * v1x + v1y * v1y;
            let dot12 = v1x * v2x + v1y * v2y;
            
            let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
            let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
            let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
            
            // Check if point is inside triangle
            if u >= 0.0 && v >= 0.0 && (u + v) <= 1.0 {
                // Anti-aliasing: fade edges (but keep center fully visible)
                let edge_dist = (u.min(v.min(1.0 - u - v)) * 2.0).min(1.0);
                let alpha_factor = edge_dist.max(0.90); // Higher minimum for better visibility
                let mut pixel_color = color;
                pixel_color.0[3] = (color.0[3] as f32 * alpha_factor) as u8;
                img.put_pixel(x, y, pixel_color);
            }
        }
    }
}

/// Generate a ring icon showing progress (0.0 = empty, 1.0 = full)
/// If paused, shows a play symbol in the center
pub fn generate_ring_icon(progress: f32, phase: Phase, is_paused: bool) -> Vec<u8> {
    let mut img: RgbaImage = ImageBuffer::new(ICON_SIZE, ICON_SIZE);

    // Determine ring color based on phase
    let ring_color = match phase {
        Phase::Work => WORK_COLOR,
        Phase::Break | Phase::LongBreak => BREAK_COLOR,
    };

    // Draw for each pixel
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let dx = x as f32 - CENTER;
            let dy = y as f32 - CENTER;
            let dist = (dx * dx + dy * dy).sqrt();

            // Check if pixel is within the ring
            let inner_radius = RADIUS - RING_WIDTH / 2.0;
            let outer_radius = RADIUS + RING_WIDTH / 2.0;

            if dist >= inner_radius && dist <= outer_radius {
                // Calculate angle from top (12 o'clock position)
                // atan2 returns angle from positive x-axis, we want from negative y-axis
                let angle = dy.atan2(dx);
                // Convert to 0..2π starting from top, going clockwise
                let normalized_angle = (angle + PI / 2.0 + 2.0 * PI) % (2.0 * PI);
                let angle_progress = normalized_angle / (2.0 * PI);

                // Anti-aliasing for ring edges
                let edge_dist_inner = (dist - inner_radius).abs();
                let edge_dist_outer = (dist - outer_radius).abs();
                let edge_dist = edge_dist_inner.min(edge_dist_outer);
                let alpha_factor = (edge_dist / 1.0).min(1.0);

                if angle_progress <= progress {
                    // Filled part of the ring
                    let mut color = ring_color;
                    color.0[3] = (color.0[3] as f32 * alpha_factor) as u8;
                    img.put_pixel(x, y, color);
                } else {
                    // Track (unfilled part)
                    let mut color = TRACK_COLOR;
                    color.0[3] = (color.0[3] as f32 * alpha_factor) as u8;
                    img.put_pixel(x, y, color);
                }
            }
        }
    }
    
    // If paused, draw play symbol in the center
    if is_paused {
        draw_play_symbol(&mut img, PLAY_SYMBOL_COLOR);
    }

    // Encode as PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    let encoder = PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(
            img.as_raw(),
            ICON_SIZE,
            ICON_SIZE,
            image::ExtendedColorType::Rgba8,
        )
        .expect("Failed to encode PNG");

    png_bytes
}

/// Create the system tray icon
/// Returns the tray icon and reference to pause/resume toggle menu item for dynamic updates
pub fn create_tray(app: &AppHandle) -> Result<(TrayIcon, MenuItem<tauri::Wry>), Box<dyn std::error::Error>> {
    // Create menu items
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let pause_resume_item = MenuItem::with_id(app, "pause_resume", "Pause", true, None::<&str>)?;
    let prefs_item = MenuItem::with_id(app, "preferences", "Preferences", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Create menu
    let menu = Menu::with_items(app, &[&show_item, &pause_resume_item, &prefs_item, &quit_item])?;

    // Generate initial icon (full ring, work phase, not paused)
    let icon_data = generate_ring_icon(1.0, Phase::Work, false);
    let icon = Image::from_bytes(&icon_data)?;

    // Build the tray icon
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Pomohardo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "pause_resume" => {
                            // Toggle pause/resume based on current timer state
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Some(state) = app_handle.try_state::<crate::AppState>() {
                                    let mut timer = state.timer.lock().await;
                                    let current_status = timer.get_state().status;
                                    match current_status {
                                        TimerStatus::Running => {
                                            timer.pause();
                                        }
                                        TimerStatus::Paused => {
                                            timer.resume();
                                        }
                                        TimerStatus::Stopped => {
                                            // Do nothing if stopped
                                        }
                                    }
                                    // Drop lock before emitting to avoid holding it during async operations (though emit is fast)
                                    drop(timer);
                                    
                                    // Trigger immediate update
                                    app_handle.emit("refresh-tray-icon", ()).ok();
                                }
                            });
                }
                "preferences" => {
                    if let Err(e) = crate::open_settings_window(app) {
                        eprintln!("Failed to open settings window: {}", e);
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok((tray, pause_resume_item))
}

/// Update the tray icon and menu based on timer state
pub async fn update_tray_icon(
    tray: &TrayIcon,
    timer: &Arc<Mutex<TimerEngine>>,
    pause_resume_item: &MenuItem<tauri::Wry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timer_guard = timer.lock().await;
    let state = timer_guard.get_state();
    drop(timer_guard);

    // Update menu item text and enabled state based on timer status
    match state.status {
        TimerStatus::Running => {
            // Show "Pause" and enable it
            let _ = pause_resume_item.set_text("Pause");
            let _ = pause_resume_item.set_enabled(true);
        }
        TimerStatus::Paused => {
            // Show "Resume" and enable it
            let _ = pause_resume_item.set_text("Resume");
            let _ = pause_resume_item.set_enabled(true);
        }
        TimerStatus::Stopped => {
            // Show "Pause" but disable it when stopped
            let _ = pause_resume_item.set_text("Pause");
            let _ = pause_resume_item.set_enabled(false);
        }
    }

    // Calculate progress (remaining / total)
    let progress = if state.total_seconds > 0 {
        state.remaining_seconds as f32 / state.total_seconds as f32
    } else {
        1.0
    };

    // Only show ring during running state
    let display_progress = if state.status == TimerStatus::Running {
        progress
    } else if state.status == TimerStatus::Paused && state.remaining_seconds == 0 {
        // Break finished, show empty ring
        0.0
    } else {
        // Stopped or paused mid-session, show current progress
        progress
    };

    // Generate new icon (show play symbol if paused)
    let is_paused = state.status == TimerStatus::Paused;
    let icon_data = generate_ring_icon(display_progress, state.phase, is_paused);
    let icon = Image::from_bytes(&icon_data)?;

    // Update tray icon
    tray.set_icon(Some(icon))?;

    Ok(())
}
