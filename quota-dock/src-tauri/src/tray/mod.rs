use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open_i = MenuItem::with_id(app, "open", "Abrir QuotaDock", true, None::<&str>)?;
    let refresh_i = MenuItem::with_id(app, "refresh", "Atualizar Agora", true, None::<&str>)?;
    let diag_i = MenuItem::with_id(app, "diagnostics", "Diagnóstico do Sistema", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_i, &refresh_i, &diag_i, &quit_i])?;

    let icon = app.default_window_icon().cloned().ok_or("Ícone padrão não encontrado")?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("QuotaDock - Monitor de Quotas IA")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app_handle, event| {
            match event.id.as_ref() {
                "open" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "refresh" => {
                    let _ = app_handle.emit("trigger-refresh", ());
                }
                "diagnostics" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = app_handle.emit("navigate-tab", "diagnostics");
                    }
                }
                "quit" => {
                    app_handle.exit(0);
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
                        // Position window near cursor / tray
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
