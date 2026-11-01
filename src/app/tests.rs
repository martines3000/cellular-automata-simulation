use super::*;
use egui::{pos2, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect};
use std::sync::Arc;

struct NoopRepaint;
impl epi::backend::RepaintSignal for NoopRepaint {
    fn request_repaint(&self) {}
}

pub(super) struct Harness {
    pub(super) app: App,
    pub(super) ctx: egui::Context,
    frame: epi::Frame,
    time: f64,
    screen_size: egui::Vec2,
}

impl Harness {
    pub(super) fn new() -> Self {
        let ctx = egui::Context::default();
        let frame = epi::Frame::new(epi::backend::FrameData {
            info: epi::IntegrationInfo {
                name: "test",
                web_info: None,
                prefer_dark_mode: Some(true),
                cpu_usage: None,
                native_pixels_per_point: Some(1.0),
            },
            output: Default::default(),
            repaint_signal: Arc::new(NoopRepaint),
        });
        let mut app = App::default();
        epi::App::setup(&mut app, &ctx, &frame, None);
        app.world.clear(); // Tests start empty; players start in the floating-ice lab.
        app.fit_view = false; // Input fixtures use a known cell scale.
        Self {
            app,
            ctx,
            frame,
            time: 0.0,
            screen_size: vec2(1280.0, 820.0),
        }
    }

    pub(super) fn input(&mut self, events: Vec<Event>) -> FullOutput {
        self.time += 0.1;
        self.ctx.begin_frame(RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), self.screen_size)),
            time: Some(self.time),
            events,
            ..Default::default()
        });
        epi::App::update(&mut self.app, &self.ctx, &self.frame);
        self.ctx.end_frame()
    }

    pub(super) fn canvas(&mut self) -> Rect {
        let mut output = self.input(vec![]);
        for _ in 0..4 {
            output = self.input(vec![]);
        }
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.1 {
                Shape::Rect(rect)
                    if rect.fill == theme::CANVAS
                        && rect.rect.width() > 200.0
                        && rect.rect.height() > 200.0 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .expect("canvas background")
    }

    pub(super) fn reveal(&mut self, label: &str) -> Pos2 {
        for _ in 0..24 {
            let output = self.input(vec![]);
            if let Some(pos) = output.shapes.iter().find_map(|shape| match &shape.1 {
                Shape::Text(text) if text.galley.text() == label => {
                    let p = text.pos + text.galley.size() * 0.5;
                    (shape.0.contains(p) && p.y < self.screen_size.y - 40.0).then_some(p)
                }
                _ => None,
            }) {
                return pos;
            }
            self.input(vec![
                Event::PointerMoved(pos2(100.0, 400.0)),
                Event::Scroll(vec2(0.0, -120.0)),
            ]);
        }
        panic!("could not reveal {label}");
    }

    pub(super) fn button(&mut self, pos: Pos2, button: PointerButton, pressed: bool) {
        self.input(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers: Modifiers::default(),
            },
        ]);
    }
}

#[test]
fn canvas_single_click_paints_once_under_pan_and_zoom() {
    for (size, shift) in [
        (5.0, vec2(0.0, 0.0)),
        (10.0, vec2(2.0, 1.0)),
        (30.0, vec2(4.0, 3.0)),
    ] {
        let mut h = Harness::new();
        h.app.world.block_size = size;
        h.app.world.shift = shift;
        h.app.world.update_pos();
        let rect = h.canvas();
        let pos = rect.min + h.app.world.pos_shift + vec2(6.5 * size, 5.5 * size);
        h.button(pos, PointerButton::Primary, true);
        h.button(pos, PointerButton::Primary, false);
        assert!(h.app.world.data[[5, 6]].cell_type == CellType::Water);
        assert_eq!(h.app.world.data[[5, 6]].mass, 1.0);
    }
}

#[test]
fn canvas_drag_paints_and_secondary_click_does_not() {
    let mut h = Harness::new();
    let rect = h.canvas();
    let start = rect.min + vec2(17.5, 17.5);
    let end = rect.min + vec2(37.5, 17.5);
    h.button(start, PointerButton::Primary, true);
    h.input(vec![Event::PointerMoved(end)]);
    h.button(end, PointerButton::Primary, false);
    assert!(h.app.world.data[[3, 7]].cell_type == CellType::Water);
    let before = h.app.world.data.clone();
    h.button(start, PointerButton::Secondary, true);
    h.button(start, PointerButton::Secondary, false);
    assert!(h.app.world.data == before);
}

#[test]
fn canvas_click_with_press_and_release_in_one_frame_paints_once() {
    let mut h = Harness::new();
    let rect = h.canvas();
    let pos = rect.min + vec2(17.5, 17.5);
    h.input(vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::default(),
        },
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::default(),
        },
    ]);
    assert!(h.app.world.data[[3, 3]].cell_type == CellType::Water);
    assert_eq!(h.app.world.data[[3, 3]].mass, 1.0);
}

#[test]
fn paused_app_stops_requesting_repaints_after_settling() {
    let mut h = Harness::new();
    for _ in 0..20 {
        h.input(vec![]);
    }
    assert!(!h.input(vec![]).needs_repaint);
    h.app.running = true;
    assert!(h.input(vec![]).needs_repaint);
    h.app.running = false;
    for _ in 0..20 {
        h.input(vec![]);
    }
    assert!(!h.input(vec![]).needs_repaint);
}

#[test]
fn canvas_reuses_shape_storage_after_submitting_each_frame() {
    let mut h = Harness::new();
    h.canvas();
    let pointer = h.app.shapes.as_ptr();
    let capacity = h.app.shapes.capacity();
    assert!(capacity > 0);
    for _ in 0..5 {
        h.input(vec![]);
        assert!(h.app.shapes.is_empty());
        assert_eq!(h.app.shapes.as_ptr(), pointer);
        assert_eq!(h.app.shapes.capacity(), capacity);
    }
}

#[test]
fn canvas_world_resize_requires_paused_confirmation() {
    let mut h = Harness::new();
    h.screen_size = vec2(1280.0, 1000.0); // Full toolbox; scrolling has its own test.
    h.canvas();
    h.app.pending_world_size = 256;
    for running in [true, false] {
        h.app.running = running;
        let output = h.input(vec![]);
        let _ = output;
        let button = h.reveal("Resize and clear");
        assert_eq!(h.app.world.data.dim(), (100, 100));
        h.button(button, PointerButton::Primary, true);
        h.button(button, PointerButton::Primary, false);
        let size = if running { 100 } else { 256 };
        assert_eq!(h.app.world.data.dim(), (size, size));
    }
    assert!(h.app.world.data == h.app.world.tmp);
    assert!(h.app.world.data.iter().all(|cell| cell.mass == 0.0));
}

fn text_position(output: &FullOutput, label: &str) -> Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.1 {
            Shape::Text(text) if text.galley.text() == label => {
                Some(text.pos + text.galley.size() * 0.5)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing UI label: {label}"))
}

#[test]
fn game_ui_palette_and_view_controls_never_paint_the_world() {
    let mut h = Harness::new();
    let rect = h.canvas();
    let before = h.app.world.data.clone();
    for &(kind, name, _, _) in &MATERIALS {
        let output = h.input(vec![]);
        let pos = text_position(&output, name);
        assert!(!rect.contains(pos));
        h.button(pos, PointerButton::Primary, true);
        h.button(pos, PointerButton::Primary, false);
        assert!(h.app.world.selected_cell_type == kind);
        assert!(h.app.world.data == before);
    }
    for label in ["Fractional surfaces", "Guide grid"] {
        let pos = text_position(&h.input(vec![]), label);
        h.button(pos, PointerButton::Primary, true);
        h.button(pos, PointerButton::Primary, false);
        assert!(h.app.world.data == before);
    }
    assert!(h.app.world.use_shift);
    assert!(h.app.show_grid);
}

#[test]
fn game_ui_fit_shows_large_world_and_preserves_materials() {
    for size in [100, 1024] {
        let mut h = Harness::new();
        h.app.world.resize_and_clear(size);
        h.app.pending_world_size = size;
        h.app.world.data[[3, 3]].cell_type = CellType::Sand;
        h.app.world.shift = vec2(4.0, 5.0);
        h.app.world.update_pos();
        h.app.fit_view = true;
        let before = h.app.world.data.clone();
        let rect = h.canvas();
        assert_eq!(h.app.world.shift, vec2(0.0, 0.0));
        assert_eq!(h.app.world.pos_shift, vec2(0.0, 0.0));
        assert!(h.app.world.block_size * size as f32 <= rect.width().min(rect.height()) + 0.001);
        assert!(h.app.world.data == before);
        assert!(h.app.fit_view);
        h.screen_size = vec2(900.0, 640.0);
        let smaller = h.canvas();
        assert!(
            h.app.world.block_size * size as f32 <= smaller.width().min(smaller.height()) + 0.001
        );
        assert!(h.app.world.data == before);
        h.app.world.shift = vec2(2.0, 2.0);
        h.app.world.update_pos();
        let pos = text_position(&h.input(vec![]), "Fit world");
        h.button(pos, PointerButton::Primary, true);
        h.button(pos, PointerButton::Primary, false);
        assert_eq!(h.app.world.shift, vec2(0.0, 0.0));
        assert!(h.app.world.data == before);
    }
}

#[test]
fn game_ui_minimum_window_keeps_palette_transport_and_canvas_separate() {
    let mut h = Harness::new();
    h.screen_size = vec2(900.0, 640.0);
    let canvas = h.canvas();
    assert!(canvas.width() > 500.0 && canvas.height() > 400.0);
    let output = h.input(vec![]);
    for label in [
        "Play", "Step", "Water", "Sand", "Dirt", "Wood", "Fire", "Eraser",
    ] {
        let pos = text_position(&output, label);
        assert!(Rect::from_min_size(Pos2::ZERO, h.screen_size).contains(pos));
        assert!(!canvas.contains(pos), "{label} overlaps canvas");
    }
}

#[test]
fn game_ui_space_toggles_play_but_respects_keyboard_focus() {
    let mut h = Harness::new();
    h.canvas();
    let key = |pressed| {
        vec![Event::Key {
            key: egui::Key::Space,
            pressed,
            modifiers: Modifiers::default(),
        }]
    };
    h.input(key(true));
    assert!(h.app.running);
    h.input(key(false));
    h.input(key(true));
    assert!(!h.app.running);
    h.input(key(false));
    h.ctx.memory().request_focus(egui::Id::new("numeric-edit"));
    assert!(h.ctx.wants_keyboard_input());
    h.input(key(true));
    assert!(!h.app.running);
}

#[test]
fn game_ui_short_window_scroll_reaches_resize_controls() {
    let mut h = Harness::new();
    h.screen_size = vec2(900.0, 640.0);
    h.app.pending_world_size = 256;
    h.canvas();
    h.input(vec![Event::PointerMoved(pos2(100.0, 400.0))]);
    h.input(vec![Event::Scroll(vec2(0.0, -1000.0))]);
    for _ in 0..20 {
        h.input(vec![]);
    }
    let output = h.input(vec![]);
    let pos = text_position(&output, "Resize and clear");
    assert!(
        pos.y > 100.0 && pos.y < 600.0,
        "resize must be reachable by scrolling: {pos:?}"
    );
    h.button(pos, PointerButton::Primary, true);
    h.button(pos, PointerButton::Primary, false);
    assert_eq!(h.app.world.data.dim(), (256, 256));
}

#[test]
fn frozen_ui_material_and_phase_tools_route_without_creating_extra_water() {
    let mut h = Harness::new();
    let rect = h.canvas();
    let cell_pos = rect.min + vec2(17.5, 17.5);
    for (label, expected, amount) in [
        ("Ice", CellType::Ice, 10.0),
        ("Melt", CellType::Water, 10.0),
        ("Freeze", CellType::Ice, 10.0),
        ("Snow", CellType::Snow, 1.0),
        ("Melt", CellType::Water, 1.0),
    ] {
        let before_selection = h.app.world.data.clone();
        let pos = text_position(&h.input(vec![]), label);
        h.button(pos, PointerButton::Primary, true);
        h.button(pos, PointerButton::Primary, false);
        assert!(h.app.world.data == before_selection);
        h.button(cell_pos, PointerButton::Primary, true);
        h.button(cell_pos, PointerButton::Primary, false);
        let cell = h.app.world.data[[3, 3]];
        assert!(cell.cell_type == expected, "tool {label}");
        assert_eq!(cell.water_quantity(), amount, "tool {label}");
    }
    let pos = text_position(&h.input(vec![]), "Sand");
    h.button(pos, PointerButton::Primary, true);
    h.button(pos, PointerButton::Primary, false);
    assert!(h.app.phase_tool.is_none());
    h.button(cell_pos, PointerButton::Primary, true);
    h.button(cell_pos, PointerButton::Primary, false);
    assert!(h.app.world.data[[3, 3]].cell_type == CellType::Sand);
}

#[test]
fn frozen_ui_brush_transforms_under_zoom_pan_and_ignores_secondary_clicks() {
    let mut h = Harness::new();
    h.app.world.block_size = 10.0;
    h.app.world.shift = vec2(2.0, 1.0);
    h.app.world.update_pos();
    h.app.world.data[[5, 6]] = world::cell::Cell {
        cell_type: CellType::Water,
        mass: 25.0,
    };
    h.app.phase_tool = Some(PhaseTool::Freeze);
    let rect = h.canvas();
    let pos = rect.min + h.app.world.pos_shift + vec2(65.0, 55.0);
    let before = h.app.world.data.clone();
    h.button(pos, PointerButton::Secondary, true);
    h.button(pos, PointerButton::Secondary, false);
    assert!(h.app.world.data == before);
    h.button(pos, PointerButton::Primary, true);
    h.button(pos, PointerButton::Primary, false);
    assert!(h.app.world.data[[5, 6]].cell_type == CellType::Ice);
    assert_eq!(h.app.world.data[[5, 6]].mass, 25.0);
    for ((y, x), cell) in h.app.world.data.indexed_iter() {
        if [y, x] != [5, 6] {
            assert!(*cell == before[[y, x]]);
        }
    }
}

#[test]
fn steam_ui_material_and_phase_tools_roundtrip_without_quantity_changes() {
    let mut h = Harness::new();
    let rect = h.canvas();
    let pos = rect.min + vec2(17.5, 17.5);
    for (label, expected) in [
        ("Steam", CellType::Steam),
        ("Condense", CellType::Water),
        ("Vaporize", CellType::Steam),
        ("Condense", CellType::Water),
        ("Freeze", CellType::Ice),
        ("Melt", CellType::Water),
        ("Vaporize", CellType::Steam),
    ] {
        let original = h.app.world.data.clone();
        let control = text_position(&h.input(vec![]), label);
        h.button(control, PointerButton::Primary, true);
        h.button(control, PointerButton::Primary, false);
        assert!(h.app.world.data == original);
        h.button(pos, PointerButton::Primary, true);
        h.button(pos, PointerButton::Primary, false);
        assert!(h.app.world.data[[3, 3]].cell_type == expected, "{label}");
        assert_eq!(h.app.world.data[[3, 3]].water_quantity(), 1.0);
    }
    let control = text_position(&h.input(vec![]), "Eraser");
    h.button(control, PointerButton::Primary, true);
    h.button(control, PointerButton::Primary, false);
    assert!(h.app.phase_tool.is_none());
    h.button(pos, PointerButton::Primary, true);
    h.button(pos, PointerButton::Primary, false);
    assert_eq!(h.app.world.data[[3, 3]].water_quantity(), 0.0);
}

#[test]
fn steam_ui_phase_brush_coalesced_clicks_respect_pan_zoom_and_material_borders() {
    let mut h = Harness::new();
    h.app.world.block_size = 10.0;
    h.app.world.shift = vec2(2.0, 1.0);
    h.app.world.update_pos();
    h.app.world.data[[5, 6]] = world::cell::Cell {
        cell_type: CellType::Water,
        mass: 25.0,
    };
    let rect = h.canvas();
    let pos = rect.min + h.app.world.pos_shift + vec2(65.0, 55.0);
    for (tool, expected) in [
        (PhaseTool::Vaporize, CellType::Steam),
        (PhaseTool::Condense, CellType::Water),
    ] {
        h.app.phase_tool = Some(tool);
        let before = h.app.world.data.clone();
        h.button(pos, PointerButton::Secondary, true);
        h.button(pos, PointerButton::Secondary, false);
        assert!(h.app.world.data == before);
        h.input(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::default(),
            },
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::default(),
            },
        ]);
        assert!(h.app.world.data[[5, 6]].cell_type == expected);
        assert_eq!(h.app.world.data[[5, 6]].mass, 25.0);
        for ((y, x), cell) in h.app.world.data.indexed_iter() {
            if [y, x] != [5, 6] {
                assert!(*cell == before[[y, x]]);
            }
        }
    }
}

#[test]
fn brush_fast_strokes_fill_all_cells_once_and_undo_restores_exact_state() {
    for square in [false, true] {
        let mut h = Harness::new();
        h.app.brush.radius = 2;
        h.app.brush.square = square;
        let view = h.canvas();
        let before = h.app.world.data.clone();
        let pos = |x: f32, y: f32| view.min + vec2(x + 0.5, y + 0.5) * 5.0;
        h.button(pos(4.0, 4.0), PointerButton::Primary, true);
        // Several pointer samples and a release may arrive in a single frame.
        h.input(vec![
            Event::PointerMoved(pos(30.0, 4.0)),
            Event::PointerMoved(pos(30.0, 30.0)),
            Event::PointerButton {
                pos: pos(30.0, 30.0),
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::default(),
            },
        ]);
        for x in 4..=30 {
            assert_eq!(h.app.world.data[[4, x]].mass, 1.0);
        }
        for y in 4..=30 {
            assert_eq!(h.app.world.data[[y, 30]].mass, 1.0);
        }
        assert!(h
            .app
            .world
            .data
            .iter()
            .all(|cell| cell.mass == 0.0 || cell.mass == 1.0));
        h.app.undo_brush();
        assert!(h.app.world.data == before);
        assert!(h.app.world.tmp == before);
        assert!(!h.app.brush.can_undo());
    }
}

#[test]
fn brush_diagonals_reverse_directions_and_max_size_preserve_borders() {
    for (start, end) in [
        ((3, 3), (40, 40)),
        ((40, 3), (3, 40)),
        ((40, 40), (3, 3)),
        ((3, 40), (40, 3)),
    ] {
        let mut h = Harness::new();
        let view = h.canvas();
        let pos = |p: (usize, usize)| view.min + vec2(p.0 as f32 + 0.5, p.1 as f32 + 0.5) * 5.0;
        h.button(pos(start), PointerButton::Primary, true);
        h.button(pos(end), PointerButton::Primary, false);
        for i in 0..=37 {
            let x = if start.0 < end.0 {
                start.0 + i
            } else {
                start.0 - i
            };
            let y = if start.1 < end.1 {
                start.1 + i
            } else {
                start.1 - i
            };
            assert_eq!(h.app.world.data[[y, x]].mass, 1.0);
        }
    }
    for square in [false, true] {
        let mut h = Harness::new();
        h.app.brush.radius = 31;
        h.app.brush.square = square;
        let view = h.canvas();
        h.button(view.min + vec2(2.5, 2.5), PointerButton::Primary, true);
        h.button(view.min + vec2(2.5, 2.5), PointerButton::Primary, false);
        for i in 0..100 {
            for index in [[0, i], [99, i], [i, 0], [i, 99]] {
                assert!(h.app.world.data[index].cell_type == CellType::Dirt);
                assert_eq!(h.app.world.data[index].mass, 0.0);
            }
        }
        assert_eq!(h.app.world.data[[1, 1]].mass, 1.0);
        assert_eq!(
            h.app.world.data[[31, 31]].mass,
            if square { 1.0 } else { 0.0 }
        );
    }
}

#[test]
fn brush_leaving_canvas_breaks_line_and_repeated_samples_do_not_pour() {
    let mut h = Harness::new();
    let view = h.canvas();
    let a = view.min + vec2(17.5, 17.5);
    let b = view.min + vec2(97.5, 17.5);
    h.button(a, PointerButton::Primary, true);
    for _ in 0..4 {
        h.input(vec![Event::PointerMoved(a)]);
    }
    h.input(vec![Event::PointerMoved(view.min - vec2(10.0, 10.0))]);
    h.input(vec![Event::PointerMoved(b)]);
    h.button(b, PointerButton::Primary, false);
    assert_eq!(h.app.world.data[[3, 3]].mass, 1.0);
    assert_eq!(h.app.world.data[[3, 19]].mass, 1.0);
    assert_eq!(h.app.world.data[[3, 10]].mass, 0.0);
    h.button(a, PointerButton::Primary, true);
    h.button(a, PointerButton::Primary, false);
    assert_eq!(h.app.world.data[[3, 3]].mass, 3.0); // A new stroke adds the existing +2 units.
    h.app.toggle_running();
    h.app.toggle_running();
    assert!(!h.app.brush.can_undo());
}

#[test]
fn brush_phase_roundtrip_and_undo_preserve_small_and_large_quantities() {
    let mut h = Harness::new();
    let view = h.canvas();
    h.app.brush.radius = 3;
    for y in 3..10 {
        for x in 3..10 {
            h.app.world.data[[y, x]] = world::cell::Cell {
                cell_type: CellType::Water,
                mass: if x % 2 == 0 { 0.005 } else { 25.0 },
            };
        }
    }
    let before = h.app.world.data.clone();
    for tool in [
        PhaseTool::Freeze,
        PhaseTool::Melt,
        PhaseTool::Vaporize,
        PhaseTool::Condense,
    ] {
        h.app.phase_tool = Some(tool);
        let pos = view.min + vec2(32.5, 32.5);
        h.button(pos, PointerButton::Primary, true);
        h.button(pos, PointerButton::Primary, false);
        for (old, new) in before.iter().zip(h.app.world.data.iter()) {
            assert_eq!(old.water_quantity(), new.water_quantity());
        }
    }
    for _ in 0..4 {
        h.app.undo_brush();
    }
    assert!(h.app.world.data == before);
}

#[test]
fn canvas_wheel_zoom_keeps_pointer_anchor_and_drag_pan_never_paints() {
    let mut h = Harness::new();
    let view = h.canvas();
    let pos = view.min + vec2(120.0, 140.0);
    h.input(vec![Event::PointerMoved(pos)]);
    let anchor = (pos - view.min - h.app.world.pos_shift) / h.app.world.block_size;
    h.input(vec![Event::Scroll(vec2(0.0, 150.0))]);
    let after = (pos - view.min - h.app.world.pos_shift) / h.app.world.block_size;
    assert!((anchor - after).length() < 0.0001);
    assert!(!h.app.fit_view);
    let before = h.app.world.data.clone();
    let shift = h.app.world.shift;
    h.button(pos, PointerButton::Secondary, true);
    h.input(vec![Event::PointerMoved(pos - vec2(30.0, 40.0))]);
    h.button(pos - vec2(30.0, 40.0), PointerButton::Secondary, false);
    assert!(h.app.world.shift.x > shift.x && h.app.world.shift.y > shift.y);
    assert!(h.app.world.data == before);
}

#[test]
fn overview_switches_to_geometry_for_fractional_surfaces_and_close_zoom() {
    let mut h = Harness::new();
    h.app.world.block_size = 1.0;
    h.canvas();
    assert!(h.app.texture.is_some());
    assert_eq!(h.app.rendered_shapes, 1);
    h.app.world.use_shift = true;
    h.input(vec![]);
    assert!(h.app.texture.is_none());
    h.app.world.use_shift = false;
    h.app.world.block_size = 5.0;
    h.input(vec![]);
    assert!(h.app.texture.is_none());
}

#[test]
fn labs_ui_requires_pause_and_clears_brush_history_on_load() {
    let mut h = Harness::new();
    let view = h.canvas();
    let p = view.min + vec2(17.5, 17.5);
    h.button(p, PointerButton::Primary, true);
    h.button(p, PointerButton::Primary, false);
    assert!(h.app.brush.can_undo());
    h.app.lab = Lab::SteamChamber;
    h.app.running = true;
    let load = text_position(&h.input(vec![]), "Load lab");
    h.button(load, PointerButton::Primary, true);
    h.button(load, PointerButton::Primary, false);
    assert!(!h
        .app
        .world
        .data
        .iter()
        .any(|cell| cell.cell_type == CellType::Ice));
    h.app.running = false;
    let load = text_position(&h.input(vec![]), "Load lab");
    h.button(load, PointerButton::Primary, true);
    h.button(load, PointerButton::Primary, false);
    assert!(h
        .app
        .world
        .data
        .iter()
        .any(|cell| cell.cell_type == CellType::Ice));
    assert!(h.app.fit_view);
    assert!(!h.app.brush.can_undo());
}

#[test]
fn canvas_zoom_out_and_repeated_frames_keep_anchor_and_do_not_pan_on_press() {
    let mut h = Harness::new();
    let view = h.canvas();
    let pos = view.min + vec2(120.0, 140.0);
    h.input(vec![Event::PointerMoved(pos)]);
    let anchor = (pos - view.min - h.app.world.pos_shift) / h.app.world.block_size;
    h.input(vec![Event::Scroll(vec2(0.0, -200.0))]);
    for _ in 0..3 {
        h.input(vec![]);
    }
    let after = (pos - view.min - h.app.world.pos_shift) / h.app.world.block_size;
    assert!((anchor - after).length() < 0.0001);
    let shift = h.app.world.shift;
    h.button(pos + vec2(200.0, 0.0), PointerButton::Secondary, true);
    assert_eq!(h.app.world.shift, shift);
    h.button(pos + vec2(200.0, 0.0), PointerButton::Secondary, false);
}

#[test]
fn canvas_pan_from_negative_zoom_shift_does_not_snap_back_to_origin() {
    let mut h = Harness::new();
    let view = h.canvas();
    h.app.world.shift = vec2(-20.0, -15.0);
    h.app.world.update_pos();
    let p = view.center();
    h.button(p, PointerButton::Secondary, true);
    h.input(vec![Event::PointerMoved(p + vec2(5.0, 5.0))]);
    assert!((h.app.world.shift - vec2(-21.0, -16.0)).length() < 0.0001);
    h.button(p + vec2(5.0, 5.0), PointerButton::Secondary, false);
}

#[test]
fn brush_undo_shortcut_restores_state_and_respects_keyboard_focus() {
    let mut h = Harness::new();
    let view = h.canvas();
    let before = h.app.world.data.clone();
    let pos = view.min + vec2(17.5, 17.5);
    h.button(pos, PointerButton::Primary, true);
    h.button(pos, PointerButton::Primary, false);
    let modifiers = Modifiers {
        command: true,
        mac_cmd: true,
        ..Default::default()
    };
    h.ctx.begin_frame(RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, h.screen_size)),
        modifiers,
        events: vec![Event::Key {
            key: egui::Key::Z,
            pressed: true,
            modifiers,
        }],
        ..Default::default()
    });
    epi::App::update(&mut h.app, &h.ctx, &h.frame);
    let _ = h.ctx.end_frame();
    assert!(h.app.world.data == before);
    assert!(!h.app.brush.can_undo());
    h.input(vec![Event::Key {
        key: egui::Key::Z,
        pressed: false,
        modifiers: Modifiers::default(),
    }]);
    h.button(pos, PointerButton::Primary, true);
    h.button(pos, PointerButton::Primary, false);
    let painted = h.app.world.data.clone();
    h.ctx.begin_frame(RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, h.screen_size)),
        modifiers,
        events: vec![Event::Key {
            key: egui::Key::Z,
            pressed: true,
            modifiers,
        }],
        ..Default::default()
    });
    h.ctx.memory().request_focus(egui::Id::new("numeric-edit"));
    assert!(h.ctx.wants_keyboard_input());
    epi::App::update(&mut h.app, &h.ctx, &h.frame);
    let _ = h.ctx.end_frame();
    assert!(h.app.world.data == painted);
}

#[test]
fn detailed_geometry_batches_touching_cells_without_antialias_edges() {
    let mut h = Harness::new();
    h.app.highlights = false;
    h.app.world.data[[4, 4]].cell_type = CellType::Dirt;
    h.app.world.data[[5, 4]].cell_type = CellType::Dirt;
    let view = h.canvas();
    let output = h.input(vec![]);
    assert!(output
        .shapes
        .iter()
        .any(|shape| matches!(&shape.1,Shape::Mesh(mesh)
        if mesh.vertices.iter().any(|v|v.color==theme::DIRT && view.contains(v.pos)))));
    assert!(!output
        .shapes
        .iter()
        .any(|shape| matches!(&shape.1,Shape::Rect(r) if r.fill==theme::DIRT && view.contains(r.rect.center()))));
}
