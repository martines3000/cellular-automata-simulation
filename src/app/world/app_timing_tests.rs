use super::*;
use crate::app::tests::Harness;
use egui::{Event, FullOutput, Modifiers, PointerButton};

fn canvas_visible(output: &FullOutput) -> bool {
    output.shapes.iter().any(|shape| {
        matches!(&shape.1, Shape::Rect(rect) if rect.fill == crate::app::theme::CANVAS
            && rect.rect.width() > 200.0 && rect.rect.height() > 200.0)
    })
}

fn settings_button(h: &mut Harness) -> Pos2 {
    h.canvas();
    h.reveal("Pan view")
}

fn pointer_button(pos: Pos2, pressed: bool) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        },
    ]
}

fn sealed_smoke() -> World {
    let mut world = World::new(7);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    world.data[[3, 3]].cell_type = CellType::Smoke;
    world.data[[3, 3]].mass = 7.0;
    world
}

#[test]
fn game_ui_running_world_advances_once_while_settings_open_and_close() {
    let mut h = Harness::new();
    let arrow = settings_button(&mut h);
    h.app.world = sealed_smoke();
    h.app.running = true;
    let mut expected = sealed_smoke();
    // Settings animation must neither suspend nor duplicate simulation updates.
    for (frame, events) in [
        vec![],
        pointer_button(arrow, true),
        pointer_button(arrow, false),
        vec![],
        vec![],
        vec![],
        pointer_button(arrow, true),
        pointer_button(arrow, false),
        vec![],
        vec![],
    ]
    .into_iter()
    .enumerate()
    {
        h.app.world.last_frame_time = Instant::now() - Duration::from_secs(100);
        let output = h.input(events);
        for _ in 0..MAX_STEPS_PER_UPDATE {
            expected.tick();
        }
        assert!(
            h.app.world.data == expected.data,
            "simulation frame {frame}"
        );
        assert!(h.app.world.tmp == expected.tmp, "scratch frame {frame}");
        assert!(output.needs_repaint);
        assert!(
            canvas_visible(&output),
            "canvas remains visible at frame {frame}"
        );
    }
}

#[test]
fn game_ui_paused_world_stays_frozen_with_settings_open_or_closed() {
    for collapsed in [false, true] {
        let mut h = Harness::new();
        let arrow = settings_button(&mut h);
        if collapsed {
            h.button(arrow, PointerButton::Primary, true);
            h.button(arrow, PointerButton::Primary, false);
        }
        for _ in 0..20 {
            h.input(vec![]);
        }
        h.app.world = sealed_smoke();
        h.app.world.last_frame_time = Instant::now() - Duration::from_secs(100);
        let before = h.app.world.data.clone();
        let clock = h.app.world.last_frame_time;
        let output = h.input(vec![]);
        assert!(h.app.world.data == before);
        assert_eq!(h.app.world.last_frame_time, clock);
        assert!(canvas_visible(&output));
        assert!(!output.needs_repaint);
    }
}

#[test]
fn game_ui_painting_happens_before_running_tick() {
    let mut h = Harness::new();
    let rect = h.canvas();
    h.app.world = World::new(7);
    h.app.world.add_border();
    h.app.world.selected_cell_type = CellType::Wood;
    h.app.running = true;
    h.app.world.last_frame_time = Instant::now() - Duration::from_secs(100);
    h.input(pointer_button(rect.min + vec2(17.5, 12.5), true));
    assert!(h.app.world.data[[2, 3]].cell_type == CellType::None);
    assert!(h.app.world.data[[5, 3]].cell_type == CellType::Wood);
}

#[test]
fn game_ui_step_advances_exactly_once_and_resets_timing() {
    let mut h = Harness::new();
    h.canvas();
    let pos = h
        .input(vec![])
        .shapes
        .iter()
        .find_map(|shape| match &shape.1 {
            Shape::Text(text) if text.galley.text() == "Step" => {
                Some(text.pos + text.galley.size() * 0.5)
            }
            _ => None,
        })
        .expect("Step control");
    h.app.world = sealed_smoke();
    h.app.world.step_credit = NANOS_PER_SECOND / 2;
    h.app.world.last_frame_time = Instant::now() - Duration::from_secs(100);
    let mut expected = sealed_smoke();
    expected.tick();
    h.button(pos, PointerButton::Primary, true);
    h.button(pos, PointerButton::Primary, false);
    assert!(!h.app.running);
    assert!(h.app.world.data == expected.data);
    assert!(h.app.world.tmp == expected.tmp);
    assert_eq!(h.app.world.step_credit, 0);
    assert!(h.app.world.last_frame_time.elapsed() < Duration::from_secs(1));
}

#[test]
fn game_ui_step_is_disabled_during_playback() {
    let mut h = Harness::new();
    h.canvas();
    let pos = h
        .input(vec![])
        .shapes
        .iter()
        .find_map(|shape| match &shape.1 {
            Shape::Text(text) if text.galley.text() == "Step" => {
                Some(text.pos + text.galley.size() * 0.5)
            }
            _ => None,
        })
        .expect("Step control");
    h.app.world = sealed_smoke();
    h.app.running = true;
    let mut expected = sealed_smoke();
    for pressed in [true, false] {
        h.app.world.last_frame_time = Instant::now() - Duration::from_secs(100);
        h.button(pos, PointerButton::Primary, pressed);
        for _ in 0..MAX_STEPS_PER_UPDATE {
            expected.tick();
        }
        assert!(h.app.world.data == expected.data);
        assert!(h.app.world.tmp == expected.tmp);
        assert!(h.app.running);
    }
}
