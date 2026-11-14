use super::super::super::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(in crate::dioxus_ui) enum Edge {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

impl Edge {
    pub fn parse(value: &str) -> Self {
        match value {
            "right" => Self::Right,
            "bottom" => Self::Bottom,
            "left" => Self::Left,
            _ => Self::Top,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }
    pub fn horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
    pub fn anchor_class(self) -> &'static str {
        match self {
            Self::Top => "top-0 left-[var(--dock-position)] [transform:translateX(var(--dock-shift))] flex-col",
            Self::Bottom => "bottom-0 left-[var(--dock-position)] [transform:translateX(var(--dock-shift))] flex-col-reverse",
            Self::Left => "left-0 top-[var(--dock-position)] [transform:translateY(var(--dock-shift))] flex-row",
            Self::Right => "right-0 top-[var(--dock-position)] [transform:translateY(var(--dock-shift))] flex-row-reverse",
        }
    }
    pub fn menu_class(self) -> &'static str {
        match self {
            Self::Top => "top-full left-[var(--dock-position)] [transform:translateX(var(--dock-shift))] mt-1",
            Self::Bottom => "bottom-full left-[var(--dock-position)] [transform:translateX(var(--dock-shift))] mb-1",
            Self::Left => "top-[var(--dock-position)] [transform:translateY(var(--dock-shift))] left-full ml-1",
            Self::Right => "top-[var(--dock-position)] [transform:translateY(var(--dock-shift))] right-full mr-1",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(in crate::dioxus_ui) struct ToolbarDock {
    pub edge: Edge,
    pub fraction: f64,
    pub dragging: bool,
}

impl ToolbarDock {
    pub fn load(session: Option<&Session<DioxusHandler>>) -> Self {
        let multi_edge = local_multi_edge();
        let option = |key: &str| {
            session.map_or_else(String::new, |session| session.get_option(key.to_owned()))
        };
        let edge = if multi_edge {
            Edge::parse(&option(OPTION_REMOTE_MENUBAR_EDGE))
        } else {
            Edge::Top
        };
        let saved = option(OPTION_REMOTE_MENUBAR_FRACTION);
        let legacy = option("remote-menubar-drag-x");
        let raw = if !multi_edge || (saved.is_empty() && edge == Edge::Top) {
            legacy
        } else {
            saved
        };
        let (left, right) = fraction_bounds(edge);
        Self {
            edge,
            fraction: raw
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .unwrap_or(0.5)
                .clamp(left, right),
            dragging: false,
        }
    }
}

pub(in crate::dioxus_ui) fn local_multi_edge() -> bool {
    crate::ui_interface::get_local_option(OPTION_ALLOW_MULTI_EDGE_TOOLBAR_DOCK.to_owned()) == "Y"
}

pub(in crate::dioxus_ui) fn fraction_bounds(edge: Edge) -> (f64, f64) {
    if !edge.horizontal() {
        return (0.0, 1.0);
    }
    let read = |key: &str, default| {
        crate::ui_interface::get_local_option(key.to_owned())
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .unwrap_or(default)
            .clamp(0.0, 1.0)
    };
    let left = read(OPTION_REMOTE_MENUBAR_DRAG_LEFT, 0.0);
    (left, read(OPTION_REMOTE_MENUBAR_DRAG_RIGHT, 1.0).max(left))
}

pub(in crate::dioxus_ui) fn drag_position(
    cursor: (f64, f64),
    parent: (f64, f64),
    size: (f64, f64),
    grab: f64,
    multi_edge: bool,
    bounds: (f64, f64),
) -> ToolbarDock {
    let edge = if multi_edge {
        [
            (Edge::Top, cursor.1),
            (Edge::Right, parent.0 - cursor.0),
            (Edge::Bottom, parent.1 - cursor.1),
            (Edge::Left, cursor.0),
        ]
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(Edge::Top, |value| value.0)
    } else {
        Edge::Top
    };
    let horizontal = edge.horizontal();
    let extent = if horizontal { parent.0 } else { parent.1 };
    let along = if horizontal { cursor.0 } else { cursor.1 };
    let long = size.0.max(size.1);
    let fraction = if extent > long {
        (along - grab.clamp(0.0, long)) / (extent - long)
    } else {
        0.5
    };
    let bounds = if horizontal { bounds } else { (0.0, 1.0) };
    ToolbarDock {
        edge,
        fraction: fraction.clamp(bounds.0, bounds.1),
        dragging: true,
    }
}

#[component]
pub(in crate::dioxus_ui) fn DockingSetting() -> Element {
    let mut enabled = use_signal(local_multi_edge);
    rsx! {
        label { class: "flex w-full max-w-[650px] items-center justify-between gap-5 border-b border-[#35373a] py-[15px] text-sm text-[#e3e3e3]",
            span { "Allow toolbar docking to all edges" }
            input { class: "size-[18px] shrink-0 accent-rd-accent", r#type: "checkbox", checked: enabled(),
                onchange: move |event| {
                    enabled.set(event.checked());
                    crate::ui_interface::set_local_option(OPTION_ALLOW_MULTI_EDGE_TOOLBAR_DOCK.to_owned(), if event.checked() { "Y" } else { "N" }.to_owned());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toolbar_drag_preserves_top_only_default_and_snaps_when_enabled() {
        let position = |cursor, multi| {
            drag_position(
                cursor,
                (1000.0, 600.0),
                (360.0, 60.0),
                180.0,
                multi,
                (0.2, 0.8),
            )
        };
        assert_eq!(position((500.0, 590.0), false).edge, Edge::Top);
        assert_eq!(position((0.0, 0.0), false).fraction, 0.2);
        assert_eq!(position((1000.0, 0.0), false).fraction, 0.8);
        assert_eq!(position((500.0, 590.0), true).edge, Edge::Bottom);
        assert_eq!(position((5.0, 300.0), true).edge, Edge::Left);
        assert_eq!(position((995.0, 300.0), true).edge, Edge::Right);
        assert_eq!(position((500.0, 5.0), true).fraction, 0.5);
    }
}
