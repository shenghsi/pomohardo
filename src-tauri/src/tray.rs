use image::{codecs::png::PngEncoder, ImageEncoder, ImageBuffer, Rgba, RgbaImage};
use std::f32::consts::PI;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
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

/// Generate a ring icon showing progress (0.0 = empty, 1.0 = full)
pub fn generate_ring_icon(progress: f32, phase: Phase) -> Vec<u8> {
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
pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    // Create menu items
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Create menu
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    // Generate initial icon (full ring, work phase)
    let icon_data = generate_ring_icon(1.0, Phase::Work);
    let icon = Image::from_bytes(&icon_data)?;

    // Build the tray icon
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Pomohardo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
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

    Ok(tray)
}

/// Update the tray icon based on timer state
pub async fn update_tray_icon(
    tray: &TrayIcon,
    timer: &Arc<Mutex<TimerEngine>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timer_guard = timer.lock().await;
    let state = timer_guard.get_state();
    drop(timer_guard);

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

    // Generate new icon
    let icon_data = generate_ring_icon(display_progress, state.phase);
    let icon = Image::from_bytes(&icon_data)?;

    // Update tray icon
    tray.set_icon(Some(icon))?;

    Ok(())
}
