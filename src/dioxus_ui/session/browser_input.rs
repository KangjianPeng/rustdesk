use super::super::prelude::*;

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(in crate::dioxus_ui) enum BrowserInput {
    Mouse {
        mask: i32,
        x: f64,
        y: f64,
        alt: bool,
        ctrl: bool,
        shift: bool,
        meta: bool,
    },
    Key {
        key: String,
        down: bool,
        alt: bool,
        ctrl: bool,
        shift: bool,
        meta: bool,
    },
    Fault {
        message: String,
    },
}

pub(in crate::dioxus_ui) fn send_browser_input(
    input: BrowserInput,
    session: &Session<DioxusHandler>,
) {
    if let BrowserInput::Fault { message } = &input {
        hbb_common::throttled_log!(Duration::from_secs(30), warn, "Remote canvas: {message}");
        return;
    }
    let release = match &input {
        BrowserInput::Mouse { mask, .. } => mask & 7 == 2,
        BrowserInput::Key { down, .. } => !down,
        BrowserInput::Fault { .. } => false,
    };
    if !release
        && (session.is_view_camera()
            || session.get_toggle_option("view-only".to_owned())
            || !*session.server_keyboard_enabled.read().unwrap())
    {
        return;
    }
    match input {
        BrowserInput::Fault { .. } => {}
        BrowserInput::Mouse {
            mask,
            x,
            y,
            alt,
            ctrl,
            shift,
            meta,
        } => {
            if !x.is_finite() || !y.is_finite() {
                return;
            }
            let (x, y) = if mask & 7 == 3 {
                (x.round() as i32, y.round() as i32)
            } else {
                let Some(state) = session.ui_handler.read_state() else {
                    return;
                };
                let display = state.displays.get(state.current_display.max(0) as usize);
                let x = (x * state.width as f64)
                    .round()
                    .clamp(0.0, state.width.saturating_sub(1).max(0) as f64)
                    as i32;
                let y = (y * state.height as f64)
                    .round()
                    .clamp(0.0, state.height.saturating_sub(1).max(0) as f64)
                    as i32;
                (
                    x.saturating_add(display.map_or(0, |d| d.x)),
                    y.saturating_add(display.map_or(0, |d| d.y)),
                )
            };
            session.send_mouse(mask, x, y, alt, ctrl, shift, meta);
        }
        BrowserInput::Key {
            key,
            down,
            alt,
            ctrl,
            shift,
            meta,
        } => {
            if let Some(key) = browser_key_name(&key) {
                session.input_key(&key, down, false, alt, ctrl, shift, meta);
            }
        }
    }
}

fn browser_key_name(key: &str) -> Option<String> {
    if key.chars().count() == 1 {
        return Some(key.to_owned());
    }
    let name = match key {
        "Backspace" => "VK_BACK",
        "Tab" => "VK_TAB",
        "Enter" => "VK_ENTER",
        "Escape" => "VK_ESCAPE",
        "Shift" => "VK_SHIFT",
        "Control" => "VK_CONTROL",
        "Alt" => "VK_MENU",
        "Meta" => "VK_LWIN",
        "CapsLock" => "VK_CAPITAL",
        "PageUp" => "VK_PRIOR",
        "PageDown" => "VK_NEXT",
        "End" => "VK_END",
        "Home" => "VK_HOME",
        "ArrowLeft" => "VK_LEFT",
        "ArrowUp" => "VK_UP",
        "ArrowRight" => "VK_RIGHT",
        "ArrowDown" => "VK_DOWN",
        "Insert" => "VK_INSERT",
        "Delete" => "VK_DELETE",
        _ => {
            return key
                .strip_prefix('F')
                .and_then(|n| n.parse::<u8>().ok())
                .filter(|n| (1..=24).contains(n))
                .map(|n| format!("VK_F{n}"))
        }
    };
    Some(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_keys_preserve_characters_modifiers_and_function_keys() {
        assert_eq!(browser_key_name(" ").as_deref(), Some(" "));
        assert_eq!(browser_key_name("中").as_deref(), Some("中"));
        assert_eq!(browser_key_name("Meta").as_deref(), Some("VK_LWIN"));
        assert_eq!(browser_key_name("ArrowDown").as_deref(), Some("VK_DOWN"));
        assert_eq!(browser_key_name("F24").as_deref(), Some("VK_F24"));
        assert!(browser_key_name("Unidentified").is_none());
    }
}
