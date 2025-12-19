// Platform-specific input blocking for BreakShield
// This module provides best-effort input blocking during breaks

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HINSTANCE;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx, HHOOK,
    WH_KEYBOARD_LL, WH_MOUSE_LL, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT,
    WINDOWS_HOOK_ID, HC_ACTION,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

pub struct InputBlocker {
    active: Arc<AtomicBool>,
    #[cfg(target_os = "linux")]
    x11: Option<X11GrabState>,
    #[cfg(target_os = "windows")]
    windows: Option<WindowsHookState>,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    // If code is HC_ACTION, we should process the event
    // Return non-zero to block the event from being passed to other applications
    if code == HC_ACTION as i32 {
        return windows::Win32::Foundation::LRESULT(1);
    }
    // Otherwise, call the next hook in the chain
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn mouse_hook_proc(
    code: i32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    // If code is HC_ACTION, we should process the event
    // Return non-zero to block the event from being passed to other applications
    if code == HC_ACTION as i32 {
        return windows::Win32::Foundation::LRESULT(1);
    }
    // Otherwise, call the next hook in the chain
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

#[cfg(target_os = "linux")]
struct X11GrabState {
    display: *mut x11::xlib::Display,
    #[allow(dead_code)]
    root: x11::xlib::Window,
}

#[cfg(target_os = "windows")]
struct WindowsHookState {
    keyboard_hook: HHOOK,
    mouse_hook: HHOOK,
}

// We only access X11GrabState behind a mutex in AppState, and we never share the raw Display*
// across threads without synchronization. Marking it Send/Sync is safe for this usage.
#[cfg(target_os = "linux")]
unsafe impl Send for X11GrabState {}
#[cfg(target_os = "linux")]
unsafe impl Sync for X11GrabState {}

#[cfg(target_os = "windows")]
unsafe impl Send for WindowsHookState {}
#[cfg(target_os = "windows")]
unsafe impl Sync for WindowsHookState {}

#[cfg(target_os = "linux")]
unsafe impl Send for InputBlocker {}
#[cfg(target_os = "linux")]
unsafe impl Sync for InputBlocker {}

#[cfg(target_os = "windows")]
unsafe impl Send for InputBlocker {}
#[cfg(target_os = "windows")]
unsafe impl Sync for InputBlocker {}

impl InputBlocker {
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            #[cfg(target_os = "linux")]
            x11: None,
            #[cfg(target_os = "windows")]
            windows: None,
        }
    }

    pub fn activate(&mut self) -> Result<(), String> {
        if self.active.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.active.store(true, Ordering::SeqCst);
        
        #[cfg(target_os = "windows")]
        {
            self.activate_windows()
        }
        
        #[cfg(target_os = "macos")]
        {
            self.activate_macos()
        }
        
        #[cfg(target_os = "linux")]
        {
            self.activate_linux()
        }
    }

    pub fn deactivate(&mut self) -> Result<(), String> {
        if !self.active.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.active.store(false, Ordering::SeqCst);
        
        #[cfg(target_os = "windows")]
        {
            self.deactivate_windows()
        }
        
        #[cfg(target_os = "macos")]
        {
            self.deactivate_macos()
        }
        
        #[cfg(target_os = "linux")]
        {
            self.deactivate_linux()
        }
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    #[cfg(target_os = "windows")]
    fn activate_windows(&mut self) -> Result<(), String> {
        // Windows implementation using low-level hooks
        // This requires SetWindowsHookEx with WH_KEYBOARD_LL and WH_MOUSE_LL
        
        unsafe {
            // Install keyboard hook
            let keyboard_hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                HINSTANCE::default(),
                0,
            ).map_err(|e| format!("Failed to install keyboard hook: {:?}", e))?;
            
            // Install mouse hook
            let mouse_hook = SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(mouse_hook_proc),
                HINSTANCE::default(),
                0,
            );
            
            // Handle partial failure: if mouse hook fails, clean up keyboard hook
            let mouse_hook = match mouse_hook {
                Ok(hook) => hook,
                Err(e) => {
                    // Clean up keyboard hook before returning error
                    let _ = UnhookWindowsHookEx(keyboard_hook);
                    return Err(format!("Failed to install mouse hook: {:?}", e));
                }
            };
            
            // Store hook handles
            self.windows = Some(WindowsHookState {
                keyboard_hook,
                mouse_hook,
            });
        }
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn deactivate_windows(&mut self) -> Result<(), String> {
        // Take ownership of the WindowsHookState to clear it
        let Some(state) = self.windows.take() else {
            // No hooks installed, nothing to do
            return Ok(());
        };
        
        unsafe {
            // Remove keyboard hook
            UnhookWindowsHookEx(state.keyboard_hook)
                .map_err(|e| format!("Failed to remove keyboard hook: {:?}", e))?;
            
            // Remove mouse hook
            UnhookWindowsHookEx(state.mouse_hook)
                .map_err(|e| format!("Failed to remove mouse hook: {:?}", e))?;
        }
        
        // WindowsHookState is automatically dropped here, clearing the struct
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn activate_macos(&self) -> Result<(), String> {
        // macOS implementation using event taps (CGEventTap)
        // This requires Accessibility permissions
        println!("macOS input blocking activated (placeholder)");
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn deactivate_macos(&self) -> Result<(), String> {
        println!("macOS input blocking deactivated (placeholder)");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn activate_linux(&mut self) -> Result<(), String> {
        // Linux X11 implementation using XGrabKeyboard/XGrabPointer
        // Wayland: overlay-only, no reliable global input blocking
        
        // Check if we're running under X11 or Wayland
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        
        if session_type == "wayland" {
            println!("Warning: Running under Wayland - input blocking limited to overlay only");
            println!("For full input blocking, use X11 session");
            return Ok(());
        }
        
        unsafe {
            let display = x11::xlib::XOpenDisplay(std::ptr::null());
            if display.is_null() {
                return Err("XOpenDisplay failed; cannot enable X11 input blocking".to_string());
            }

            let root = x11::xlib::XDefaultRootWindow(display);

            // Grab keyboard. owner_events=true so input still goes to the focused window (our fullscreen app).
            let kb = x11::xlib::XGrabKeyboard(
                display,
                root,
                x11::xlib::True,
                x11::xlib::GrabModeAsync,
                x11::xlib::GrabModeAsync,
                x11::xlib::CurrentTime,
            );
            if kb != x11::xlib::GrabSuccess {
                x11::xlib::XCloseDisplay(display);
                return Err(format!("XGrabKeyboard failed with code {}", kb));
            }

            // Grab pointer (mouse). owner_events=true so clicks/moves still go to focused window.
            let event_mask = (x11::xlib::ButtonPressMask
                | x11::xlib::ButtonReleaseMask
                | x11::xlib::PointerMotionMask) as u32;

            let ptr = x11::xlib::XGrabPointer(
                display,
                root,
                x11::xlib::True,
                event_mask,
                x11::xlib::GrabModeAsync,
                x11::xlib::GrabModeAsync,
                0,
                0,
                x11::xlib::CurrentTime,
            );

            if ptr != x11::xlib::GrabSuccess {
                // Undo keyboard grab
                x11::xlib::XUngrabKeyboard(display, x11::xlib::CurrentTime);
                x11::xlib::XCloseDisplay(display);
                return Err(format!("XGrabPointer failed with code {}", ptr));
            }

            x11::xlib::XFlush(display);
            self.x11 = Some(X11GrabState { display, root });
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn deactivate_linux(&mut self) -> Result<(), String> {
        let Some(state) = self.x11.take() else {
            return Ok(());
        };

        unsafe {
            x11::xlib::XUngrabPointer(state.display, x11::xlib::CurrentTime);
            x11::xlib::XUngrabKeyboard(state.display, x11::xlib::CurrentTime);
            x11::xlib::XFlush(state.display);
            x11::xlib::XCloseDisplay(state.display);
        }
        Ok(())
    }

    /// Linux/X11 only: check whether Ctrl+Alt+Shift+E is currently pressed.
    /// This is used because X11 grabs can prevent the webview from receiving key events.
    #[cfg(target_os = "linux")]
    pub fn emergency_chord_pressed(&mut self) -> Result<bool, String> {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        if session_type == "wayland" {
            return Ok(false);
        }

        // If we don't already have a display, open a temporary one for querying.
        let (display, close_after) = match self.x11 {
            Some(ref s) => (s.display, false),
            None => unsafe {
                let d = x11::xlib::XOpenDisplay(std::ptr::null());
                if d.is_null() {
                    return Err("XOpenDisplay failed; cannot query emergency chord".to_string());
                }
                (d, true)
            },
        };

        unsafe {
            // Query keymap
            let mut keys: [i8; 32] = [0; 32];
            x11::xlib::XQueryKeymap(display, keys.as_mut_ptr());

            let pressed = |kc: u8| -> bool {
                let idx = (kc / 8) as usize;
                let bit = kc % 8;
                (keys[idx] & (1 << bit)) != 0
            };

            let kc_ctrl_l = x11::xlib::XKeysymToKeycode(display, x11::keysym::XK_Control_L as u64);
            let kc_ctrl_r = x11::xlib::XKeysymToKeycode(display, x11::keysym::XK_Control_R as u64);
            let kc_alt_l = x11::xlib::XKeysymToKeycode(display, x11::keysym::XK_Alt_L as u64);
            let kc_alt_r = x11::xlib::XKeysymToKeycode(display, x11::keysym::XK_Alt_R as u64);
            let kc_shift_l = x11::xlib::XKeysymToKeycode(display, x11::keysym::XK_Shift_L as u64);
            let kc_shift_r = x11::xlib::XKeysymToKeycode(display, x11::keysym::XK_Shift_R as u64);
            let kc_e = x11::xlib::XKeysymToKeycode(display, x11::keysym::XK_E as u64);

            let ctrl = pressed(kc_ctrl_l) || pressed(kc_ctrl_r);
            let alt = pressed(kc_alt_l) || pressed(kc_alt_r);
            let shift = pressed(kc_shift_l) || pressed(kc_shift_r);
            let e = pressed(kc_e);

            if close_after {
                x11::xlib::XCloseDisplay(display);
            }

            Ok(ctrl && alt && shift && e)
        }
    }
}

impl Default for InputBlocker {
    fn default() -> Self {
        Self::new()
    }
}

