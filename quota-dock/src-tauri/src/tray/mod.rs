use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
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
fn should_hide_on_blur(pinned: bool, foreground: bool) -> bool {
    !pinned && !foreground
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Hidden,
    Opening,
    Visible,
    Closing,
    Minimized,
}
struct Transition {
    token: u32,
    phase: Phase,
}
impl Default for Transition {
    fn default() -> Self {
        Self {
            token: 0,
            phase: Phase::Hidden,
        }
    }
}
impl Transition {
    fn open(&mut self) -> u32 {
        self.token = self.token.wrapping_add(1);
        self.phase = Phase::Opening;
        self.token
    }
    fn close(&mut self) -> Option<u32> {
        if !matches!(self.phase, Phase::Visible | Phase::Opening) {
            return None;
        }
        self.token = self.token.wrapping_add(1);
        self.phase = Phase::Closing;
        Some(self.token)
    }
    fn accepts(&self, token: u32, phase: Phase) -> bool {
        self.token == token && self.phase == phase
    }
}
#[derive(Default)]
pub struct FlyoutState {
    transition: Mutex<Transition>,
    pinned: AtomicBool,
    taskbar: AtomicBool,
    dragged: AtomicBool,
}
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub opening_token: Option<u32>,
    pub pinned: bool,
    pub show_in_taskbar: bool,
}
pub fn window_state(app: &AppHandle) -> WindowState {
    let state = app.state::<FlyoutState>();
    let t = state.transition.lock().expect("window transition");
    WindowState {
        opening_token: if matches!(t.phase, Phase::Opening | Phase::Visible) {
            Some(t.token)
        } else {
            None
        },
        pinned: state.pinned.load(Ordering::Relaxed),
        show_in_taskbar: state.taskbar.load(Ordering::Relaxed),
    }
}
pub fn apply_preferences(
    window: &WebviewWindow,
    pinned: bool,
    taskbar: bool,
) -> Result<(), tauri::Error> {
    let state = window.app_handle().state::<FlyoutState>();
    window.set_always_on_top(pinned)?;
    if let Err(error) = window.set_skip_taskbar(!taskbar) {
        let _ = window.set_always_on_top(state.pinned.load(Ordering::Relaxed));
        return Err(error);
    }
    state.pinned.store(pinned, Ordering::Relaxed);
    state.taskbar.store(taskbar, Ordering::Relaxed);
    if !taskbar {
        let was_minimized = {
            let mut t = state.transition.lock().expect("window transition");
            let minimized = t.phase == Phase::Minimized;
            if minimized { t.phase = Phase::Hidden; }
            minimized
        };
        if was_minimized { window.hide()?; }
    }
    let _ = window
        .app_handle()
        .emit("window-preferences", window_state(window.app_handle()));
    Ok(())
}
pub fn start_minimized(window: &WebviewWindow) -> Result<(), tauri::Error> {
    let state = window.app_handle().state::<FlyoutState>();
    if state.taskbar.load(Ordering::Relaxed) {
        window.minimize()?;
        window.show()?;
        state.transition.lock().expect("window transition").phase = Phase::Minimized;
    }
    Ok(())
}
pub fn restore_from_taskbar(window: &WebviewWindow) {
    let minimized = window
        .app_handle()
        .state::<FlyoutState>()
        .transition
        .lock()
        .expect("window transition")
        .phase
        == Phase::Minimized;
    if minimized && !window.is_minimized().unwrap_or(true) {
        let _ = show_flyout(window.app_handle(), None);
    }
}
pub fn show_ready(window: &WebviewWindow, token: u32) -> Result<bool, tauri::Error> {
    let state = window.app_handle().state::<FlyoutState>();
    let mut t = state.transition.lock().expect("window transition");
    if t.token != token || !matches!(t.phase, Phase::Opening | Phase::Visible) {
        return Ok(false);
    }
    t.phase = Phase::Visible;
    drop(t); // Native focus callbacks can run synchronously on Windows.
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(true)
}
pub fn finish_hide(window: &WebviewWindow, token: u32) -> Result<bool, tauri::Error> {
    let state = window.app_handle().state::<FlyoutState>();
    let mut t = state.transition.lock().expect("window transition");
    if !t.accepts(token, Phase::Closing) {
        return Ok(false);
    }
    let taskbar = state.taskbar.load(Ordering::Relaxed);
    t.phase = if taskbar { Phase::Minimized } else { Phase::Hidden };
    drop(t);
    if taskbar {
        window.minimize()?;
    } else {
        window.hide()?;
    }
    Ok(true)
}
#[cfg(windows)]
fn cursor_message(x: i32, y: i32) -> isize {
    ((x as u16 as u32) | ((y as u16 as u32) << 16)) as isize
}
pub fn start_dragging(window: &WebviewWindow) -> Result<(), String> {
    window
        .app_handle()
        .state::<FlyoutState>()
        .dragged
        .store(true, Ordering::Relaxed);
    #[cfg(windows)]
    {
        use windows_sys::Win32::{
            Foundation::POINT,
            UI::{
                Input::KeyboardAndMouse::ReleaseCapture,
                WindowsAndMessaging::{GetCursorPos, PostMessageW, HTCAPTION, WM_NCLBUTTONDOWN},
            },
        };
        let hwnd = window.hwnd().map_err(|_| "Falha ao acessar janela")?;
        let mut point = POINT { x: 0, y: 0 };
        // Native WM_NCLBUTTONDOWN requires packed coordinates, not a pointer to
        // stack memory. Avoid the upstream Tao 0.37 Windows drag regression.
        unsafe {
            if GetCursorPos(&mut point) == 0 {
                return Err("Falha ao iniciar arrasto".into());
            }
            ReleaseCapture();
            if PostMessageW(
                hwnd.0 as _,
                WM_NCLBUTTONDOWN,
                HTCAPTION as usize,
                cursor_message(point.x, point.y),
            ) == 0
            {
                return Err("Falha ao iniciar arrasto".into());
            }
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        window
            .start_dragging()
            .map_err(|_| "Falha ao iniciar arrasto".into())
    }
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
        let height = flyout_height(
            requested,
            work.size.height.saturating_sub(frame),
            m.scale_factor(),
        );
        if inner.height.abs_diff(height) <= 1 {
            return Ok(());
        }
        let pos = window.outer_position()?;
        let gap = (12.0 * m.scale_factor()).round() as i32;
        let outer_height = height.saturating_add(frame);
        let desired_y = if window
            .app_handle()
            .state::<FlyoutState>()
            .dragged
            .load(Ordering::Relaxed)
        {
            pos.y
        } else {
            work.position.y + (work.size.height as i32 - outer_height as i32) / 2
        };
        let y = desired_y.clamp(
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
fn is_foreground(window: &WebviewWindow) -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetAncestor, GetForegroundWindow, GA_ROOT};
        let Ok(hwnd) = window.hwnd() else { return false; };
        unsafe { GetAncestor(GetForegroundWindow(), GA_ROOT) == hwnd.0 as _ }
    }
    #[cfg(not(windows))]
    { window.is_focused().unwrap_or(false) }
}
pub fn hide_on_blur(window: &WebviewWindow) {
    let win = window.clone();
    tauri::async_runtime::spawn(async move {
        // Caption dragging transfers focus from WebView to its native parent.
        // Let Windows finish that transfer before checking the foreground root.
        tokio::time::sleep(Duration::from_millis(50)).await;
        let on_main = win.clone();
        let _ = win.run_on_main_thread(move || hide_flyout(&on_main, true));
    });
}
pub fn hide_flyout(window: &WebviewWindow, automatic: bool) {
    let state = window.app_handle().state::<FlyoutState>();
    if automatic && !should_hide_on_blur(state.pinned.load(Ordering::Relaxed), is_foreground(window)) {
        return;
    }
    if window.is_minimized().unwrap_or(false) {
        return;
    }
    let token = state.transition.lock().expect("window transition").close();
    let Some(token) = token else {
        return;
    };
    if let Ok(mut time) = HIDDEN.get_or_init(|| Mutex::new(None)).lock() {
        *time = Some(Instant::now());
    }
    let _ = window.app_handle().emit("flyout-closing", token);
    let win = window.clone();
    tauri::async_runtime::spawn(async move {
        // The renderer normally finishes the 170ms animation first. This deadline
        // also recollects the native window if WebView rendering is suspended.
        tokio::time::sleep(Duration::from_millis(260)).await;
        let on_main = win.clone();
        let _ = win.run_on_main_thread(move || {
            // Validate the token on the same thread as the native operation;
            // a queued fallback must not hide a more recent opening.
            let _ = finish_hide(&on_main, token);
        });
    });
}
pub fn flyout_position(
    work: (i32, i32, u32, u32),
    size: (u32, u32),
    _anchor: (i32, i32),
    gap: i32,
) -> (i32, i32) {
    let (x, y, w, h) = work;
    let right = x + w as i32;
    let bottom = y + h as i32;
    let px = (right - size.0 as i32 - gap).max(x + gap);
    let py = (y + (h as i32 - size.1 as i32) / 2)
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
    let state = app.state::<FlyoutState>();
    if state.transition.lock().expect("window transition").phase == Phase::Visible
        && window.is_visible()?
        && !window.is_minimized()?
    {
        window.set_focus()?;
        return Ok(());
    }
    state.dragged.store(false, Ordering::Relaxed);
    let token = state.transition.lock().expect("window transition").open();
    let monitor = if let Some(p) = anchor {
        window.monitor_from_point(p.x as f64, p.y as f64)?
    } else {
        window.current_monitor()?.or(window.primary_monitor()?)
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
    app.emit("flyout-opening", token)?;
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
                    if window.is_visible().unwrap_or(false)
                        && !window.is_minimized().unwrap_or(false)
                    {
                        hide_flyout(&window, false);
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
    #[test]
    fn reopening_cancels_a_pending_hide() {
        let mut t = Transition::default();
        let first = t.open();
        assert!(t.accepts(first, Phase::Opening));
        t.phase = Phase::Visible;
        let closing = t.close().unwrap();
        let reopened = t.open();
        assert!(!t.accepts(closing, Phase::Closing));
        assert!(t.accepts(reopened, Phase::Opening));
        assert!(!t.accepts(first, Phase::Opening));
    }
    #[test]
    fn pinned_window_stays_visible_when_another_app_gets_focus() {
        assert!(!should_hide_on_blur(true, false));
        assert!(should_hide_on_blur(false, false));
    }
    #[test]
    fn native_caption_focus_transfer_does_not_hide_an_unpinned_window() {
        assert!(!should_hide_on_blur(false, true));
    }
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
            (1548, 300)
        );
        let (x, y) = flyout_position((-1920, 0, 1920, 1040), (360, 440), (-1700, 1000), 12);
        assert!(x >= -1908 && x + 360 <= -12);
        assert!(y >= 12 && y + 440 <= 1028);
    }
}
