use super::*;
use crate::app::tests::Harness;
use egui::{Event, FullOutput, Modifiers, PointerButton};

fn canvas_visible(output: &FullOutput) -> bool {
    output.shapes.iter().any(|shape| {
        matches!(&shape.1, Shape::Rect(rect) if rect.fill == Color32::WHITE
            && rect.rect.width() > 200.0 && rect.rect.height() > 200.0)
    })
}

fn collapse_button(h: &mut Harness) -> Pos2 {
    let canvas = h.canvas();
    h.input(vec![])
        .shapes
        .iter()
        .find_map(|shape| match &shape.1 {
            Shape::Path(path) if path.points.len() == 3 => {
                let center = Rect::from_points(&path.points).center();
                (center.x >= canvas.left()
                    && center.x < canvas.left() + 30.0
                    && center.y < canvas.top()
                    && center.y > canvas.top() - 40.0)
                    .then_some(center)
            }
            _ => None,
        })
        .expect("Display collapse arrow")
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
fn display_visibility_running_world_advances_through_collapse_and_reopen() {
    let mut h = Harness::new();
    let arrow = collapse_button(&mut h);
    h.app.world = sealed_smoke();
    h.app.running = true;
    let mut expected = sealed_smoke();
    // Include all animation frames, fully collapsed frames, and reopening.
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
        if frame == 5 {
            assert!(
                !canvas_visible(&output),
                "fixture must fully collapse Display"
            );
        } else if frame == 9 {
            assert!(canvas_visible(&output), "fixture must reopen Display");
        }
    }
}

#[test]
fn display_visibility_paused_world_stays_frozen_in_both_states() {
    for collapsed in [false, true] {
        let mut h = Harness::new();
        let arrow = collapse_button(&mut h);
        if collapsed {
            h.button(arrow, PointerButton::Primary, true);
            h.button(arrow, PointerButton::Primary, false);
        }
        for _ in 0..5 {
            h.input(vec![]);
        }
        h.app.world = sealed_smoke();
        h.app.world.last_frame_time = Instant::now() - Duration::from_secs(100);
        let before = h.app.world.data.clone();
        let clock = h.app.world.last_frame_time;
        let output = h.input(vec![]);
        assert!(h.app.world.data == before);
        assert_eq!(h.app.world.last_frame_time, clock);
        assert_eq!(canvas_visible(&output), !collapsed);
        assert!(!output.needs_repaint);
    }
}

#[test]
fn display_visibility_painting_happens_before_running_tick() {
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
