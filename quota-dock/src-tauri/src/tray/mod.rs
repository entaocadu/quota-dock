use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex, OnceLock,
};
use std::time::{Duration, Instant};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow,
};

static HIDDEN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static HEIGHT: AtomicU32 = AtomicU32::new(240);
#[cfg(windows)]
fn cursor_message(x: i32, y: i32) -> isize {
    ((x as u16 as u32) | ((y as u16 as u32) << 16)) as isize
}
pub fn start_dragging(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::{Foundation::POINT, UI::{Input::KeyboardAndMouse::ReleaseCapture, WindowsAndMessaging::{GetCursorPos, PostMessageW, WM_NCLBUTTONDOWN, HTCAPTION}}};
        let hwnd = window.hwnd().map_err(|_| "Falha ao acessar janela")?;
        let mut point = POINT { x: 0, y: 0 };
        // Native WM_NCLBUTTONDOWN requires packed coordinates, not a pointer to
        // stack memory. Avoid the upstream Tao 0.37 Windows drag regression.
        unsafe {
            if GetCursorPos(&mut point) == 0 { return Err("Falha ao iniciar arrasto".into()); }
            ReleaseCapture();
            if PostMessageW(hwnd.0 as _, WM_NCLBUTTONDOWN, HTCAPTION as usize, cursor_message(point.x, point.y)) == 0 {
                return Err("Falha ao iniciar arrasto".into());
            }
        }
        Ok(())
    }
    #[cfg(not(windows))]
    { window.start_dragging().map_err(|_| "Falha ao iniciar arrasto".into()) }
}
fn flyout_height(requested: f64, available: u32, scale: f64) -> u32 {
    let gap = (12.0 * scale).round() as u32;
    ((requested.clamp(160.0, 640.0) * scale).ceil() as u32)
        .min(available.saturating_sub(gap * 2).max(1))
}
pub fn resize_flyout(window: &WebviewWindow, requested: f64) -> Result<(), tauri::Error> {
    if !requested.is_finite() {
        return Ok(());
    }
    HEIGHT.store(
        requested.clamp(160.0, 640.0).ceil() as u32,
        Ordering::Relaxed,
    );
    if let Some(m) = window.current_monitor()? {
        let work = m.work_area();
        let inner = window.inner_size()?;
        let old = window.outer_size()?;
        let frame = old.height.saturating_sub(inner.height);
        let height = flyout_height(requested, work.size.height.saturating_sub(frame), m.scale_factor());
        if inner.height.abs_diff(height) <= 1 {
            return Ok(());
        }
        let pos = window.outer_position()?;
        let gap = (12.0 * m.scale_factor()).round() as i32;
        let outer_height = height.saturating_add(frame);
        let y = (pos.y + old.height as i32 - outer_height as i32).clamp(
            work.position.y + gap,
            (work.position.y + work.size.height as i32 - outer_height as i32 - gap)
                .max(work.position.y + gap),
        );
        // Tauri set_size uses client dimensions; outer dimensions include the
        // invisible Windows frame and would accumulate on each observer callback.
        window.set_size(PhysicalSize::new(inner.width, height))?;
        window.set_position(PhysicalPosition::new(pos.x, y))?;
    }
    Ok(())
}
pub fn hide_flyout(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        if let Ok(mut time) = HIDDEN.get_or_init(|| Mutex::new(None)).lock() {
            *time = Some(Instant::now());
        }
    }
}
pub fn flyout_position(
    work: (i32, i32, u32, u32),
    size: (u32, u32),
    anchor: (i32, i32),
    gap: i32,
) -> (i32, i32) {
    let (x, y, w, h) = work;
    let right = x + w as i32;
    let bottom = y + h as i32;
    let px =
        (anchor.0 - size.0 as i32 / 2).clamp(x + gap, (right - size.0 as i32 - gap).max(x + gap));
    let py = (anchor.1 - size.1 as i32 - gap)
        .clamp(y + gap, (bottom - size.1 as i32 - gap).max(y + gap));
    (px, py)
}
pub fn show_flyout(
    app: &AppHandle,
    anchor: Option<PhysicalPosition<i32>>,
) -> Result<(), tauri::Error> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    let monitor = if let Some(p) = anchor {
        window.monitor_from_point(p.x as f64, p.y as f64)?
    } else {
        window.primary_monitor()?
    };
    if let Some(m) = monitor {
        let work = m.work_area();
        let gap = (12.0 * m.scale_factor()).round() as i32;
        let size = PhysicalSize::new(
            (360.0 * m.scale_factor()).round() as u32,
            flyout_height(
                HEIGHT.load(Ordering::Relaxed) as f64,
                work.size.height,
                m.scale_factor(),
            ),
        );
        window.set_size(size)?;
        let outer = window.outer_size()?;
        let anchor = anchor.unwrap_or(PhysicalPosition::new(
            work.position.x + work.size.width as i32 - (180.0 * m.scale_factor()) as i32 - gap,
            work.position.y + work.size.height as i32,
        ));
        let (x, y) = flyout_position(
            (
                work.position.x,
                work.position.y,
                work.size.width,
                work.size.height,
            ),
            (outer.width, outer.height),
            (anchor.x, anchor.y),
            gap,
        );
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    window.show()?;
    window.set_focus()?;
    let _ = app.emit("flyout-opened", ());
    Ok(())
}
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open = MenuItem::with_id(app, "open", "Abrir QuotaDock", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Atualizar quotas", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &refresh, &quit])?;
    TrayIconBuilder::with_id("main-tray")
        .tooltip("QuotaDock · quotas do OpenCode")
        .icon(
            app.default_window_icon()
                .cloned()
                .ok_or("Ícone não encontrado")?,
        )
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = show_flyout(app, None);
            }
            "refresh" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::refresh(&handle, true).await;
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        hide_flyout(&window);
                    } else {
                        // A tray click first blurs the webview. Prevent that same click reopening it.
                        let just_hidden = HIDDEN
                            .get_or_init(|| Mutex::new(None))
                            .lock()
                            .ok()
                            .and_then(|t| *t)
                            .is_some_and(|t| t.elapsed() < Duration::from_millis(250));
                        if !just_hidden {
                            let _ = show_flyout(
                                app,
                                Some(PhysicalPosition::new(position.x as i32, position.y as i32)),
                            );
                        }
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn drag_message_preserves_signed_monitor_coordinates() {
        let packed = cursor_message(-100, 200) as u32;
        assert_eq!(packed as u16 as i16, -100);
        assert_eq!((packed >> 16) as u16 as i16, 200);
    }
    #[test]
    fn fits_one_or_a_hundred_accounts_without_covering_the_screen() {
        assert_eq!(flyout_height(180.0, 1040, 1.0), 180);
        assert_eq!(flyout_height(14000.0, 1040, 1.0), 640);
        assert_eq!(flyout_height(14000.0, 400, 1.0), 376);
        assert_eq!(flyout_height(180.0, 2080, 2.0), 360);
    }
    #[test]
    fn clamps_popup_to_work_area_and_supports_negative_monitor_coordinates() {
        assert_eq!(
            flyout_position((0, 0, 1920, 1040), (360, 440), (1910, 1040), 12),
            (1548, 588)
        );
        let (x, y) = flyout_position((-1920, 0, 1920, 1040), (360, 440), (-1700, 1000), 12);
        assert!(x >= -1908 && x + 360 <= -12);
        assert!(y >= 12 && y + 440 <= 1028);
    }
}
