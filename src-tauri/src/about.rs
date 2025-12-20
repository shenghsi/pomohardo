use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const ABOUT_WINDOW_WIDTH: f64 = 450.0;
pub const ABOUT_WINDOW_HEIGHT: f64 = 300.0;

pub fn open_about_window(app: &AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("about") {
        let _ = w.close();
    }

    WebviewWindowBuilder::new(app, "about", WebviewUrl::App("about.html".into()))
        .title("About Pomohardo")
        .inner_size(ABOUT_WINDOW_WIDTH, ABOUT_WINDOW_HEIGHT)
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
