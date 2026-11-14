use super::super::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub(in crate::dioxus_ui) struct RemoteCursorState {
    shapes: HashMap<u64, Arc<CursorData>>,
    selected: Option<u64>,
    pub position: Option<(i32, i32)>,
}

impl RemoteCursorState {
    pub fn store(&mut self, data: CursorData) {
        self.selected = Some(data.id);
        self.shapes.insert(data.id, Arc::new(data));
    }

    pub fn select(&mut self, id: &str) {
        self.selected = id.parse().ok();
    }

    pub fn shape(&self) -> Option<Arc<CursorData>> {
        self.shapes.get(&self.selected?).cloned()
    }

    pub fn overlay_rect(
        &self,
        display: &DisplayInfo,
        output: (u32, u32),
        pixel_ratio: f64,
        zoom: bool,
    ) -> Option<[f32; 4]> {
        if display.cursor_embedded || display.width <= 0 || display.height <= 0 {
            return None;
        }
        let (x, y) = self.position?;
        let x = (x as f64 - display.x as f64) / display.width as f64;
        let y = (y as f64 - display.y as f64) / display.height as f64;
        if !(0.0..1.0).contains(&x) || !(0.0..1.0).contains(&y) {
            return None;
        }
        let shape = self.shape()?;
        let (sx, sy) = if zoom {
            (1.0 / display.width as f64, 1.0 / display.height as f64)
        } else {
            (
                pixel_ratio / output.0.max(1) as f64,
                pixel_ratio / output.1.max(1) as f64,
            )
        };
        Some([
            (x - shape.hotx as f64 * sx) as f32,
            (y - shape.hoty as f64 * sy) as f32,
            (shape.width as f64 * sx) as f32,
            (shape.height as f64 * sy) as f32,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_cursor_selection_and_hotspot_follow_the_current_display() {
        let mut cursor = RemoteCursorState::default();
        cursor.store(CursorData {
            id: 7,
            width: 20,
            height: 10,
            hotx: 2,
            hoty: 3,
            ..Default::default()
        });
        cursor.store(CursorData {
            id: 8,
            width: 10,
            height: 20,
            ..Default::default()
        });
        cursor.select("7");
        assert_eq!(cursor.shape().unwrap().hotx, 2);
        cursor.position = Some((-1400, 100));
        let mut display = DisplayInfo {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        let rect = cursor
            .overlay_rect(&display, (960, 540), 1.0, true)
            .unwrap();
        assert!((rect[0] - 518.0 / 1920.0).abs() < 0.0001);
        assert!((rect[1] - 97.0 / 1080.0).abs() < 0.0001);
        assert!((rect[2] - 20.0 / 1920.0).abs() < 0.0001);
        display.cursor_embedded = true;
        assert!(cursor
            .overlay_rect(&display, (960, 540), 1.0, true)
            .is_none());
        cursor.select("99");
        assert!(cursor.shape().is_none());
    }
}
