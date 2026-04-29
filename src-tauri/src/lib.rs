mod usage;

use notify_debouncer_mini::new_debouncer;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

#[cfg(target_os = "macos")]
fn set_macos_accessory_app() {
    use objc2::class;
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    unsafe {
        let cls = class!(NSApplication);
        let app: *mut AnyObject = msg_send![cls, sharedApplication];
        // NSApplicationActivationPolicyAccessory = 1
        // No Dock icon, no Cmd-Tab entry. macOS no longer manages our
        // window with the regular Space mechanics, which is the only
        // configuration where the 'stationary' collection behavior is
        // honored reliably.
        let _: () = msg_send![app, setActivationPolicy: 1i64];
    }
}

#[cfg(not(target_os = "macos"))]
fn set_macos_accessory_app() {}

#[cfg(target_os = "macos")]
fn set_macos_panel_behavior(window: &WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let Ok(ns_window) = window.ns_window() else {
        return;
    };
    let ns_window = ns_window as *mut AnyObject;
    if ns_window.is_null() {
        return;
    }
    // Pet behavior we want:
    //   - canJoinAllSpaces (1<<0)        appear on every Space
    //   - stationary       (1<<4)        don't slide with Space switches
    //   - ignoresCycle     (1<<6)        skip Cmd-` cycling
    //   - fullScreenAuxiliary (1<<8)     overlay fullscreen apps
    let behavior: u64 = (1 << 0) | (1 << 4) | (1 << 6) | (1 << 8);
    // kCGFloatingWindowLevelKey-equivalent that survives Mission Control:
    // NSStatusWindowLevel = 25.
    let level: i64 = 25;
    unsafe {
        let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
        let _: () = msg_send![ns_window, setLevel: level];
        let _: () = msg_send![ns_window, setHidesOnDeactivate: false];
        // Disable the per-Space slide animation entirely. Without this the
        // window can still briefly travel during the transition even with
        // 'stationary' set — NSWindowAnimationBehaviorNone freezes that.
        // NSWindowAnimationBehaviorNone = 2
        let _: () = msg_send![ns_window, setAnimationBehavior: 2i64];
        // setMovableByWindowBackground stays true (we drag from the panda).
    }
}

#[cfg(not(target_os = "macos"))]
fn set_macos_panel_behavior(_window: &WebviewWindow) {}

struct WatcherState {
    _debouncer: Mutex<Option<Box<dyn std::any::Any + Send>>>,
}

#[tauri::command]
fn get_usage_snapshot() -> usage::UsageSnapshot {
    usage::snapshot()
}

#[tauri::command]
fn claude_projects_path() -> Option<PathBuf> {
    usage::claude_projects_dir()
}

#[tauri::command]
fn set_tray_title(app: AppHandle, title: String) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_title(Some(title)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn toggle_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn emit_snapshot(app: &AppHandle) {
    let snap = usage::snapshot();
    let _ = app.emit("usage-update", &snap);
}

fn start_watcher(app: AppHandle) -> Arc<WatcherState> {
    let state = Arc::new(WatcherState {
        _debouncer: Mutex::new(None),
    });

    let Some(root) = usage::claude_projects_dir() else {
        log::warn!("~/.claude/projects not found — watcher idle");
        return state;
    };

    let app_for_events = app.clone();
    let mut debouncer = match new_debouncer(
        Duration::from_millis(500),
        move |res: notify_debouncer_mini::DebounceEventResult| match res {
            Ok(_events) => emit_snapshot(&app_for_events),
            Err(e) => log::error!("watch error: {:?}", e),
        },
    ) {
        Ok(d) => d,
        Err(e) => {
            log::error!("failed to create debouncer: {:?}", e);
            return state;
        }
    };

    if let Err(e) = debouncer
        .watcher()
        .watch(&root, notify::RecursiveMode::Recursive)
    {
        log::error!("failed to watch {:?}: {:?}", root, e);
        return state;
    }

    *state._debouncer.lock() = Some(Box::new(debouncer));

    let app_for_tick = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        emit_snapshot(&app_for_tick);
        loop {
            std::thread::sleep(Duration::from_secs(15));
            emit_snapshot(&app_for_tick);
        }
    });

    state
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "펫 보이기/숨기기", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "설정...", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &settings_item, &quit_item])?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .title("🐼 …")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = app.emit("show-settings", ());
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // Hide the Dock icon FIRST so the window we're about to attach
            // panel-behavior to is created under accessory mode.
            set_macos_accessory_app();

            let handle = app.handle().clone();
            build_tray(&handle)?;

            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                {
                    // 1) Apply once during setup.
                    set_macos_panel_behavior(&window);

                    // 2) Apply again ~200ms later — tao re-applies its own
                    //    collection-behavior bits during early window
                    //    lifecycle, after our setup hook has returned.
                    let w_for_thread = window.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(200));
                        let w_for_main = w_for_thread.clone();
                        let _ = w_for_thread.run_on_main_thread(move || {
                            set_macos_panel_behavior(&w_for_main);
                        });
                    });

                    // 3) Re-apply on every relevant lifecycle event. Some
                    //    tao/macOS interactions (focus, Space change, app
                    //    activation) reset the collection behavior; we
                    //    enforce it back to our values each time.
                    let w_for_event = window.clone();
                    window.on_window_event(move |event| {
                        use tauri::WindowEvent;
                        match event {
                            WindowEvent::Focused(_)
                            | WindowEvent::Resized(_)
                            | WindowEvent::Moved(_) => {
                                let w = w_for_event.clone();
                                let _ = w_for_event.run_on_main_thread(move || {
                                    set_macos_panel_behavior(&w);
                                });
                            }
                            _ => {}
                        }
                    });
                }
            }

            let watcher = start_watcher(handle);
            app.manage(watcher);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usage_snapshot,
            claude_projects_path,
            set_tray_title,
            toggle_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
