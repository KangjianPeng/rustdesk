use super::super::prelude::*;

fn mouse_button_mask(button: MouseButton) -> i32 {
    match button {
        MouseButton::Primary => 1,
        MouseButton::Secondary => 2,
        MouseButton::Auxiliary => 4,
        MouseButton::Fourth => 8,
        MouseButton::Fifth => 16,
        MouseButton::Unknown => 0,
    }
}

fn held_button_mask(buttons: dioxus::html::input_data::MouseButtonSet) -> i32 {
    let mut mask = 0;
    for button in [
        MouseButton::Primary,
        MouseButton::Secondary,
        MouseButton::Auxiliary,
        MouseButton::Fourth,
        MouseButton::Fifth,
    ] {
        if buttons.contains(button) {
            mask |= mouse_button_mask(button);
        }
    }
    mask
}

pub(crate) fn remote_point(x: f64, y: f64, state: &DioxusUiState) -> (i32, i32) {
    let scale_x = if state.viewport_width > 1.0 && state.width > 0 {
        state.width as f64 / state.viewport_width
    } else {
        1.0
    };
    let scale_y = if state.viewport_height > 1.0 && state.height > 0 {
        state.height as f64 / state.viewport_height
    } else {
        1.0
    };
    let x = (x * scale_x).round();
    let y = (y * scale_y).round();
    let max_x = state.width.saturating_sub(1).max(0) as f64;
    let max_y = state.height.saturating_sub(1).max(0) as f64;
    (x.clamp(0.0, max_x) as i32, y.clamp(0.0, max_y) as i32)
}

pub(crate) fn send_mouse_event(
    data: &MouseData,
    event_type: i32,
    session: &Session<DioxusHandler>,
    state: &DioxusUiState,
) {
    let point = data.element_coordinates();
    let (x, y) = remote_point(point.x, point.y, state);
    let buttons = data
        .trigger_button()
        .map(mouse_button_mask)
        .unwrap_or_else(|| held_button_mask(data.held_buttons()));
    let modifiers = data.modifiers();
    session.send_mouse(
        buttons << 3 | event_type,
        x,
        y,
        modifiers.alt(),
        modifiers.ctrl(),
        modifiers.shift(),
        modifiers.meta(),
    );
}

fn touch_point(data: &TouchData, state: &DioxusUiState) -> Option<(i32, i32)> {
    let point = data
        .touches_changed()
        .into_iter()
        .next()
        .or_else(|| data.touches().into_iter().next())?;
    let client = point.client_coordinates();
    Some(remote_point(
        client.x - state.viewport_left,
        client.y - state.viewport_top,
        state,
    ))
}

pub(crate) fn send_touch_event(
    data: &TouchData,
    event_type: i32,
    session: &Session<DioxusHandler>,
    state: &DioxusUiState,
) {
    let Some((x, y)) = touch_point(data, state) else {
        return;
    };
    let modifiers = data.modifiers();
    session.send_mouse(
        1 << 3 | event_type,
        x,
        y,
        modifiers.alt(),
        modifiers.ctrl(),
        modifiers.shift(),
        modifiers.meta(),
    );
}

pub(crate) fn send_wheel_event(data: &WheelData, session: &Session<DioxusHandler>) {
    let delta = data.delta().strip_units();
    let modifiers = data.modifiers();
    session.send_mouse(
        3,
        delta.x.round() as i32,
        delta.y.round() as i32,
        modifiers.alt(),
        modifiers.ctrl(),
        modifiers.shift(),
        modifiers.meta(),
    );
}

fn keyboard_key_name(data: &KeyboardData) -> Option<String> {
    let key = data.key().to_string();
    if key.chars().count() == 1 {
        return Some(key);
    }
    let name = match key.as_str() {
        "Backspace" => "VK_BACK",
        "Tab" => "VK_TAB",
        "Enter" => "VK_ENTER",
        "Escape" => "VK_ESCAPE",
        "Shift" => "VK_SHIFT",
        "Control" => "VK_CONTROL",
        "Alt" => "VK_MENU",
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
        "F1" => "VK_F1",
        "F2" => "VK_F2",
        "F3" => "VK_F3",
        "F4" => "VK_F4",
        "F5" => "VK_F5",
        "F6" => "VK_F6",
        "F7" => "VK_F7",
        "F8" => "VK_F8",
        "F9" => "VK_F9",
        "F10" => "VK_F10",
        "F11" => "VK_F11",
        "F12" => "VK_F12",
        " " => "VK_SPACE",
        _ => return None,
    };
    Some(name.to_owned())
}

pub(crate) fn send_keyboard_event(
    data: &KeyboardData,
    down: bool,
    session: &Session<DioxusHandler>,
) {
    let Some(name) = keyboard_key_name(data) else {
        return;
    };
    let modifiers = data.modifiers();
    session.input_key(
        &name,
        down,
        false,
        modifiers.alt(),
        modifiers.ctrl(),
        modifiers.shift(),
        modifiers.meta(),
    );
}
