//! Fork (sh4wty1/whatRust): give memory back while WhatsApp is out of sight (Windows).
//!
//! 1. A WhatsApp webview that is not on screen asks WebView2 for its low memory
//!    target (`ICoreWebView2_19::MemoryUsageTargetLevel`), and for the normal one again
//!    when it is shown. Scripts keep running, so notifications and the unread count
//!    are unaffected. (`TrySuspend` would save more but pauses them.)
//! 2. The account bar `shell-main` and the settings panel are the only webviews of the
//!    separate "management" WebView2 profile, which runs its own browser, GPU, network
//!    and storage processes. Once the main window has stayed hidden (tray, lock) for
//!    [`IDLE_BEFORE_CLOSE`], both are closed so that profile exits; the bar is rebuilt
//!    the next time the window is shown. A short hide never pays the rebuild.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};

const IDLE_BEFORE_CLOSE: Duration = Duration::from_secs(60);
/// The rebuilt bar loads asynchronously; refresh its account list once it has.
const SHELL_SETTLE: Duration = Duration::from_secs(2);

/// Whether the main window was hidden at the last layout pass.
static HIDDEN: AtomicBool = AtomicBool::new(false);
/// Bumped on every hidden/shown transition; a pending close only runs if it is unchanged.
static EPOCH: AtomicU64 = AtomicU64::new(0);
static REBUILDING: AtomicBool = AtomicBool::new(false);

/// Ask WebView2 for its low (or normal) memory usage target for `view`.
pub fn set_target(view: &tauri::Webview, low: bool) {
    let _ = view.with_webview(move |platform| {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        };
        use windows_core::Interface;
        let level = if low {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
        } else {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
        };
        // SAFETY: with_webview runs on the UI thread that owns the controller. On a
        // runtime older than ICoreWebView2_19 the cast fails and nothing changes.
        unsafe {
            let Ok(core) = platform.controller().CoreWebView2() else {
                return;
            };
            if let Ok(core) = core.cast::<ICoreWebView2_19>() {
                let _ = core.SetMemoryUsageTargetLevel(level);
            }
        }
    });
}

/// Called by every layout pass of the main window (UI thread).
pub fn main_window(app: &AppHandle, hidden: bool) {
    let was_hidden = HIDDEN.swap(hidden, Ordering::SeqCst);
    if !hidden {
        if was_hidden {
            EPOCH.fetch_add(1, Ordering::SeqCst);
        }
        ensure_shell(app);
        return;
    }
    if was_hidden {
        return;
    }
    let epoch = EPOCH.fetch_add(1, Ordering::SeqCst) + 1;
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(IDLE_BEFORE_CLOSE);
        let ui = handle.clone();
        let _ = handle.run_on_main_thread(move || close_management(&ui, epoch));
    });
}

fn close_management(app: &AppHandle, epoch: u64) {
    if EPOCH.load(Ordering::SeqCst) != epoch || !HIDDEN.load(Ordering::SeqCst) {
        return;
    }
    // An open panel is on screen somewhere (or must survive the hide): keep the profile.
    if crate::sessions::panel_open(app) {
        return;
    }
    let mut closed = Vec::new();
    for label in ["shell-main", "settings"] {
        if let Some(view) = app.get_webview(label) {
            if view.close().is_ok() {
                crate::sessions::forget_view_geometry(app, label);
                closed.push(label);
            }
        }
    }
    if !closed.is_empty() {
        crate::dlog::log(&format!(
            "memory: closed {} after {}s hidden",
            closed.join(", "),
            IDLE_BEFORE_CLOSE.as_secs()
        ));
    }
}

/// Rebuild the account bar if a tray stay closed it. Never blocks the UI thread:
/// creating a WebView2 from it while it dispatches the layout would wait on itself.
fn ensure_shell(app: &AppHandle) {
    if app.get_webview("shell-main").is_some() || REBUILDING.swap(true, Ordering::SeqCst) {
        return;
    }
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = match handle.get_window("main") {
            Some(w) if handle.get_webview("shell-main").is_none() => {
                crate::sessions::add_shell(&handle, &w).map(|_| true)
            }
            _ => Ok(false),
        };
        REBUILDING.store(false, Ordering::SeqCst);
        match result {
            Ok(true) => {
                crate::dlog::log("memory: rebuilt shell-main");
                crate::sessions::layout(&handle, "main");
                std::thread::sleep(SHELL_SETTLE);
                crate::integration::broadcast(&handle);
            }
            Ok(false) => {}
            Err(e) => crate::dlog::log(&format!("memory: shell-main rebuild failed: {e}")),
        }
    });
}
