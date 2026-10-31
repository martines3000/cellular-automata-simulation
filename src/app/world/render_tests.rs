use super::*;

fn rectangles(world: &World, viewport: Rect) -> Vec<(Rect, Color32)> {
    let mut shapes = Vec::new();
    world.gen_shapes(&mut shapes, viewport);
    shapes
        .into_iter()
        .map(|shape| match shape {
            Shape::Rect(rect) => (rect.rect, rect.fill),
            _ => panic!("cell renderer must emit rectangles"),
        })
        .collect()
}

#[test]
fn rendering_empty_world_has_no_cell_geometry() {
    let world = World::new(100);
    assert!(rectangles(
        &world,
        Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 600.0))
    )
    .is_empty());
}

#[test]
fn rendering_only_includes_visible_cells_under_pan() {
    let mut world = World::new(100);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    let viewport = Rect::from_min_size(pos2(100.0, 200.0), vec2(20.0, 15.0));
    assert_eq!(rectangles(&world, viewport).len(), 12);
    world.pos_shift = vec2(-2.5, -2.5);
    assert_eq!(rectangles(&world, viewport).len(), 20);
    for shift in [vec2(-2000.0, -2000.0), vec2(2000.0, 2000.0)] {
        world.pos_shift = shift;
        assert!(rectangles(&world, viewport).is_empty());
    }
}

#[test]
fn rendering_water_and_wood_keep_fractional_surface_geometry() {
    let mut world = World::new(7);
    world.block_size = 10.0;
    world.use_shift = true;
    world.data[[1, 2]].cell_type = CellType::Wood;
    world.data[[2, 2]].cell_type = CellType::Water;
    world.data[[2, 2]].mass = 5.0;
    // Preserve the legacy treatment of a water-bearing None cell under water.
    world.data[[3, 2]].mass = 0.2;
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(70.0, 70.0));
    assert_eq!(
        rectangles(&world, viewport),
        vec![
            (
                Rect::from_min_max(pos2(20.0, 15.0), pos2(30.0, 25.0)),
                Color32::BROWN
            ),
            (
                Rect::from_min_max(pos2(20.0, 25.0), pos2(30.0, 30.0)),
                Color32::BLUE
            ),
            (
                Rect::from_min_max(pos2(20.0, 30.0), pos2(30.0, 40.0)),
                Color32::BLUE
            ),
        ]
    );
    world.pos_shift = vec2(-20.0, -20.0);
    assert_eq!(
        rectangles(
            &world,
            Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0))
        ),
        vec![
            (
                Rect::from_min_max(pos2(0.0, -5.0), pos2(10.0, 5.0)),
                Color32::BROWN
            ),
            (
                Rect::from_min_max(pos2(0.0, 5.0), pos2(10.0, 10.0)),
                Color32::BLUE
            ),
        ]
    );
}

#[test]
fn rendering_rejects_empty_viewports_and_invalid_scales() {
    let mut world = World::new(7);
    world.data[[2, 2]].cell_type = CellType::Dirt;
    for size in [vec2(0.0, 50.0), vec2(50.0, 0.0), vec2(-1.0, 50.0)] {
        assert!(rectangles(&world, Rect::from_min_size(pos2(0.0, 0.0), size)).is_empty());
    }
    for size in [0.0, -1.0, f32::INFINITY, f32::NAN] {
        world.block_size = size;
        assert!(rectangles(
            &world,
            Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0))
        )
        .is_empty());
    }
}

#[test]
fn rendering_border_materials_do_not_index_outside_the_grid() {
    let mut world = World::new(3);
    world.use_shift = true;
    world.data[[0, 1]].cell_type = CellType::Water;
    world.data[[0, 1]].mass = 20.0;
    world.data[[2, 1]].cell_type = CellType::Wood;
    assert_eq!(
        rectangles(
            &world,
            Rect::from_min_size(pos2(0.0, 0.0), vec2(15.0, 15.0))
        )
        .len(),
        2
    );
}
