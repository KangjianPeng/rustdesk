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
            Self::Top => "grid-cols-[minmax(0,var(--dock-before))_minmax(0,max-content)_minmax(0,var(--dock-after))] grid-rows-[auto_minmax(0,1fr)]",
            Self::Bottom => "grid-cols-[minmax(0,var(--dock-before))_minmax(0,max-content)_minmax(0,var(--dock-after))] grid-rows-[minmax(0,1fr)_auto]",
            Self::Left => "grid-rows-[minmax(0,var(--dock-before))_minmax(0,max-content)_minmax(0,var(--dock-after))] grid-cols-[auto_minmax(0,1fr)]",
            Self::Right => "grid-rows-[minmax(0,var(--dock-before))_minmax(0,max-content)_minmax(0,var(--dock-after))] grid-cols-[minmax(0,1fr)_auto]",
        }
    }
    pub fn menu_class(self, fraction: f64) -> &'static str {
        match self {
            Self::Top => "top-full mt-1",
            Self::Bottom => "bottom-full mb-1",
            Self::Left if fraction <= 0.5 => "top-0 left-full ml-1",
            Self::Left => "bottom-0 left-full ml-1",
            Self::Right if fraction <= 0.5 => "top-0 right-full mr-1",
            Self::Right => "bottom-0 right-full mr-1",
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

#[derive(serde::Deserialize)]
struct DragEvent {
    cursor: (f64, f64),
    parent: (f64, f64),
    size: (f64, f64),
    grab: f64,
    finished: bool,
}

pub(super) fn install_drag(mut dock: SyncSignal<ToolbarDock>) {
    let mut eval = document::eval(include_str!("toolbar_drag.js"));
    spawn(async move {
        while let Ok(event) = eval.recv::<DragEvent>().await {
            let mut next = drag_position(
                event.cursor,
                event.parent,
                event.size,
                event.grab,
                local_multi_edge(),
                fraction_bounds(Edge::Top),
            );
            next.dragging = !event.finished;
            dock.set(next);
            if event.finished {
                if let Some(session) = crate::dioxus_ui::current_session() {
                    session.set_option(
                        OPTION_REMOTE_MENUBAR_EDGE.to_owned(),
                        next.edge.name().to_owned(),
                    );
                    session.set_option(
                        OPTION_REMOTE_MENUBAR_FRACTION.to_owned(),
                        next.fraction.to_string(),
                    );
                    if !local_multi_edge() {
                        session.set_option(
                            "remote-menubar-drag-x".to_owned(),
                            next.fraction.to_string(),
                        );
                    }
                }
            }
        }
    });
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

    #[test]
    fn side_toolbar_menus_open_toward_the_available_vertical_space() {
        assert_eq!(Edge::Left.menu_class(0.25), "top-0 left-full ml-1");
        assert_eq!(Edge::Left.menu_class(0.75), "bottom-0 left-full ml-1");
        assert_eq!(Edge::Right.menu_class(0.25), "top-0 right-full mr-1");
        assert_eq!(Edge::Right.menu_class(0.75), "bottom-0 right-full mr-1");
    }
}
