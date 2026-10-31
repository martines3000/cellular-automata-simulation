use super::*;
use egui::{pos2, Event, FullOutput, Modifiers, PointerButton, RawInput, Rect};
use std::sync::Arc;

struct NoopRepaint;
impl epi::backend::RepaintSignal for NoopRepaint {
    fn request_repaint(&self) {}
}

struct Harness {
    app: App,
    ctx: egui::Context,
    frame: epi::Frame,
    time: f64,
}

impl Harness {
    fn new() -> Self {
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
        Self {
            app,
            ctx,
            frame,
            time: 0.0,
        }
    }

    fn input(&mut self, events: Vec<Event>) -> FullOutput {
        self.time += 0.1;
        self.ctx.begin_frame(RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(2000.0, 1200.0))),
            time: Some(self.time),
            events,
            ..Default::default()
        });
        epi::App::update(&mut self.app, &self.ctx, &self.frame);
        self.ctx.end_frame()
    }

    fn canvas(&mut self) -> Rect {
        let mut output = self.input(vec![]);
        for _ in 0..4 {
            output = self.input(vec![]);
        }
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.1 {
                Shape::Rect(rect)
                    if rect.fill == Color32::WHITE
                        && rect.rect.width() > 200.0
                        && rect.rect.height() > 200.0 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .expect("canvas background")
    }

    fn button(&mut self, pos: Pos2, button: PointerButton, pressed: bool) {
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
