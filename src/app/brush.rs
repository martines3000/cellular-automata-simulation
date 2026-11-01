use super::{world::cell::Cell, App};
use egui::{Event, PointerButton, Pos2, Rect};
use std::collections::{HashSet, VecDeque};

const MAX_UNDO_BYTES: usize = 32 * 1024 * 1024;
const MAX_UNDO_STROKES: usize = 20;
type Changes = Vec<(usize, Cell)>;

#[derive(Default)]
pub(super) struct Brushes {
    pub radius: i32,
    pub square: bool,
    stroke: Option<Stroke>,
    undo: VecDeque<Changes>,
}

#[derive(Default)]
struct Stroke {
    previous: Option<[i32; 2]>,
    seen: HashSet<usize>,
    before: Changes,
}

impl Brushes {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn clear(&mut self) {
        self.stroke = None;
        self.undo.clear();
    }
    fn finish(&mut self) {
        if let Some(stroke) = self.stroke.take() {
            if !stroke.before.is_empty() {
                self.undo.push_back(stroke.before);
            }
        }
        while self.undo.len() > MAX_UNDO_STROKES
            || self
                .undo
                .iter()
                .map(|s| s.capacity() * std::mem::size_of::<(usize, Cell)>())
                .sum::<usize>()
                > MAX_UNDO_BYTES
        {
            self.undo.pop_front();
        }
    }
}

impl App {
    pub(super) fn undo_brush(&mut self) {
        if self.running {
            return;
        }
        self.brush.finish();
        if let Some(changes) = self.brush.undo.pop_back() {
            let cols = self.world.data.ncols();
            for (index, cell) in changes {
                self.world.data[[index / cols, index % cols]] = cell;
            }
            self.world.tmp.assign(&self.world.data);
            self.world.reset_timing();
        }
    }

    pub(super) fn brush_input(
        &mut self,
        events: &[Event],
        owns_pointer: bool,
        rect: Rect,
        primary_down: bool,
    ) {
        for event in events {
            match *event {
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    ..
                } if owns_pointer => {
                    self.brush.finish();
                    self.brush.stroke = Some(Stroke::default());
                    self.brush_point(pos, rect);
                }
                Event::PointerMoved(pos) if self.brush.stroke.is_some() => {
                    self.brush_point(pos, rect)
                }
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    ..
                } => {
                    if self.brush.stroke.is_some() {
                        self.brush_point(pos, rect);
                    }
                    self.brush.finish();
                }
                Event::PointerGone => {
                    self.brush.finish();
                }
                _ => {}
            }
        }
        if !primary_down {
            self.brush.finish();
        }
    }

    fn brush_point(&mut self, pos: Pos2, rect: Rect) {
        let size = self.world.data.ncols() as i32;
        let cell = ((pos - rect.min - self.world.pos_shift) / self.world.block_size).floor();
        let valid = rect.contains(pos)
            && pos.x < rect.max.x
            && pos.y < rect.max.y
            && cell.is_finite()
            && cell.x >= 0.0
            && cell.y >= 0.0
            && cell.x < size as f32
            && cell.y < size as f32;
        let stroke = self.brush.stroke.as_mut().unwrap();
        if !valid {
            stroke.previous = None;
            return;
        }
        let end = [cell.x as i32, cell.y as i32];
        let start = stroke.previous.replace(end).unwrap_or(end);
        let radius = self.brush.radius.clamp(0, 31);
        let square = self.brush.square;
        // Bresenham joins samples in cell coordinates, independent of event rate.
        let [mut x, mut y] = start;
        let dx = (end[0] - x).abs();
        let dy = -(end[1] - y).abs();
        let sx = if x < end[0] { 1 } else { -1 };
        let sy = if y < end[1] { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            for by in (y - radius).max(1)..=(y + radius).min(size - 2) {
                for bx in (x - radius).max(1)..=(x + radius).min(size - 2) {
                    if !square && (bx - x).pow(2) + (by - y).pow(2) > radius.pow(2) {
                        continue;
                    }
                    let linear = by as usize * size as usize + bx as usize;
                    if !stroke.seen.insert(linear) {
                        continue;
                    }
                    let index = [by as usize, bx as usize];
                    let before = self.world.data[index];
                    if let Some(tool) = self.phase_tool {
                        self.world.phase_cell(index, tool);
                    } else {
                        self.world.paint_cell(index);
                    }
                    if !self.running && self.world.data[index] != before {
                        stroke.before.push((linear, before));
                    }
                }
            }
            if [x, y] == end {
                break;
            }
            let twice_error = 2 * error;
            if twice_error >= dy {
                error += dy;
                x += sx;
            }
            if twice_error <= dx {
                error += dx;
                y += sy;
            }
        }
    }

    pub(super) fn navigate(&mut self, ctx: &egui::Context, response: &egui::Response, rect: Rect) {
        if self.brush.stroke.is_some() || ctx.input().pointer.primary_down() {
            return;
        }
        let pan_pressed = ctx.input().events.iter().any(|event| {
            matches!(
                event,
                Event::PointerButton {
                    button: PointerButton::Middle | PointerButton::Secondary,
                    pressed: true,
                    ..
                }
            )
        });
        if !pan_pressed
            && (response.dragged_by(PointerButton::Middle)
                || response.dragged_by(PointerButton::Secondary))
        {
            self.world.shift -= ctx.input().pointer.delta() / self.world.block_size;
            let max = self.world.data.ncols() as f32;
            self.world.shift.x = self.world.shift.x.clamp(-max, max);
            self.world.shift.y = self.world.shift.y.clamp(-max, max);
            self.world.update_pos();
            self.fit_view = false;
        }
        if response.hovered() && self.brush.stroke.is_none() {
            if let Some(pos) = response.hover_pos() {
                let scroll = ctx.input().scroll_delta.y;
                let zoom = ctx.input().zoom_delta();
                let factor = (scroll * 0.003).exp() * zoom;
                if factor.is_finite() && factor > 0.0 && factor != 1.0 {
                    let anchor = (pos - rect.min - self.world.pos_shift) / self.world.block_size;
                    let scale = (self.world.block_size * factor).clamp(0.25, 30.0);
                    // Allow negative shift here so zooming out retains the pointer anchor.
                    self.world.shift = anchor - (pos - rect.min) / scale;
                    self.world.block_size = scale;
                    self.world.update_pos();
                    self.fit_view = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_history_bounds_stroke_count_and_memory() {
        let mut brush = Brushes::default();
        for _ in 0..25 {
            brush.stroke = Some(Stroke {
                before: vec![(0, Cell::default())],
                ..Default::default()
            });
            brush.finish();
        }
        assert_eq!(brush.undo.len(), MAX_UNDO_STROKES);
        brush.clear();
        let count = MAX_UNDO_BYTES / std::mem::size_of::<(usize, Cell)>();
        brush.stroke = Some(Stroke {
            before: vec![(0, Cell::default()); count],
            ..Default::default()
        });
        brush.finish();
        brush.stroke = Some(Stroke {
            before: vec![(1, Cell::default())],
            ..Default::default()
        });
        brush.finish();
        assert_eq!(brush.undo.len(), 1);
        assert_eq!(brush.undo.front().unwrap().len(), 1);
        let mut reserved = Vec::with_capacity(count + 1);
        reserved.push((2, Cell::default()));
        brush.stroke = Some(Stroke {
            before: reserved,
            ..Default::default()
        });
        brush.finish();
        assert!(
            brush.undo.is_empty(),
            "count retained capacity, including unused slots"
        );
    }
}
