use super::super::super::prelude::*;
use super::menus::{Divider, MenuItem};

#[component]
pub(super) fn ResolutionMenu(
    display: DisplayInfo,
    resolutions: Vec<Resolution>,
    local_resolution: Option<(i32, i32)>,
    disabled: bool,
    onchange: EventHandler<(i32, i32)>,
) -> Element {
    let scale = if display.scale.is_finite() && display.scale > 0.0 {
        display.scale
    } else {
        1.0
    };
    let current = (
        (display.width as f64 / scale) as i32,
        (display.height as f64 / scale) as i32,
    );
    let supported = |size: (i32, i32)| {
        size.0 > 0
            && size.1 > 0
            && resolutions
                .iter()
                .any(|resolution| (resolution.width, resolution.height) == size)
    };
    let original = (
        display.original_resolution.width,
        display.original_resolution.height,
    );
    let restore = current != original && supported(original);
    let fit = local_resolution.filter(|size| current != *size && supported(*size));
    rsx! {
        if restore {
            MenuItem { label: format!("Original resolution · {} × {}", original.0, original.1), disabled,
                onclick: move |_| onchange.call(original)
            }
        }
        if let Some(size) = fit {
            MenuItem { label: format!("Fit local resolution · {} × {}", size.0, size.1), disabled,
                onclick: move |_| onchange.call(size)
            }
        }
        if restore || fit.is_some() { Divider {} }
        for resolution in resolutions.iter().filter(|resolution| resolution.width > 0 && resolution.height > 0) {
            {
                let size = (resolution.width, resolution.height);
                rsx! {
                    MenuItem { label: format!("{} × {}", size.0, size.1), radio: true, checked: current == size, disabled,
                        onclick: move |_| if current != size { onchange.call(size); }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::toolbar::ToolbarState;
    use super::*;

    #[test]
    fn resolution_menu_marks_scaled_remote_size_and_offers_supported_shortcuts() {
        fn fixture() -> Element {
            let toolbar = ToolbarState {
                menu: use_signal(|| Some("display")),
                submenu: use_signal(|| Some("resolution")),
                revision: use_signal(|| 0),
                quality_dialog: use_signal(|| false),
            };
            use_context_provider(|| toolbar);
            let resolution = |width, height| Resolution {
                width,
                height,
                ..Default::default()
            };
            rsx! { ResolutionMenu {
                display: DisplayInfo { width: 1920, height: 1080, scale: 1.5, original_resolution: Some(resolution(1920, 1080)).into(), ..Default::default() },
                resolutions: vec![resolution(1280, 720), resolution(1920, 1080)],
                local_resolution: Some((1920, 1080)), disabled: false,
                onchange: |_| {},
            } }
        }
        let mut dom = VirtualDom::new(fixture);
        let mutations = dom.rebuild_to_vec();
        let text = mutations
            .edits
            .iter()
            .filter_map(|edit| match edit {
                dioxus::dioxus_core::Mutation::CreateTextNode { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Original resolution · 1920 × 1080"), "{text}");
        assert!(
            text.contains("Fit local resolution · 1920 × 1080"),
            "{text}"
        );
        assert!(text.contains("1280 × 720"), "{text}");
        let selected = mutations.edits.iter().filter(|edit| matches!(edit,
            dioxus::dioxus_core::Mutation::SetAttribute { name: "aria-checked", value: dioxus::dioxus_core::AttributeValue::Text(value), .. } if value == "true"
        )).count();
        assert_eq!(selected, 1);
    }
}
