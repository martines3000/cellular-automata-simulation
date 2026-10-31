use super::*;

#[test]
fn rendering_dense_basin_has_linear_geometry() {
    let counts: Vec<_> = [100, 256, 512, 1024]
        .into_iter()
        .map(|size| {
            let world = fixtures::basin_world(size);
            let viewport =
                Rect::from_min_size(pos2(0.0, 0.0), Vec2::splat(size as f32 * world.block_size));
            (size, rectangles(&world, viewport).len())
        })
        .collect();
    eprintln!("Full basin rectangle counts: {counts:?}");
    assert!(counts.iter().all(|(size, count)| *count <= 3 * size));
}

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
    for (shift, expected_runs) in [(vec2(0.0, 0.0), 3), (vec2(-2.5, -2.5), 4)] {
        world.pos_shift = shift;
        let shapes = rectangles(&world, viewport);
        assert_eq!(shapes.len(), expected_runs);
        let area: f32 = shapes
            .iter()
            .map(|(rect, _)| {
                let clipped = rect.intersect(viewport);
                clipped.width() * clipped.height()
            })
            .sum();
        assert_eq!(area, viewport.width() * viewport.height());
    }
    for shift in [vec2(-2000.0, -2000.0), vec2(2000.0, 2000.0)] {
        world.pos_shift = shift;
        assert!(rectangles(&world, viewport).is_empty());
    }
}

#[test]
fn rendering_runs_preserve_gaps_colors_and_caller_shapes() {
    let mut world = World::new(7);
    for (x, material) in [
        (1, CellType::Dirt),
        (2, CellType::Dirt),
        (4, CellType::Sand),
        (5, CellType::Water),
    ] {
        world.data[[2, x]].cell_type = material;
    }
    world.data[[2, 5]].mass = MAX_MASS;
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(35.0, 35.0));
    let marker = Shape::rect_filled(
        Rect::from_min_max(pos2(0.0, 10.0), pos2(5.0, 15.0)),
        Rounding::none(),
        Color32::BLACK,
    );
    let mut shapes = vec![marker.clone()];
    world.gen_shapes(&mut shapes, viewport);
    assert_eq!(shapes[0], marker); // Appending must not extend the caller's rectangle.
    assert_eq!(
        rectangles(&world, viewport),
        vec![
            (
                Rect::from_min_max(pos2(5.0, 10.0), pos2(15.0, 15.0)),
                Color32::BLACK
            ),
            (
                Rect::from_min_max(pos2(20.0, 10.0), pos2(25.0, 15.0)),
                Color32::GOLD
            ),
            (
                Rect::from_min_max(pos2(25.0, 10.0), pos2(30.0, 15.0)),
                Color32::BLUE
            ),
        ]
    );
    assert_eq!(shapes.len(), 4);
}

#[test]
fn rendering_runs_keep_fractional_heights_and_merge_only_equal_surfaces() {
    for right_mass in [5.0, 10.0] {
        let mut world = World::new(7);
        world.block_size = 10.0;
        world.use_shift = true;
        for (x, mass) in [(2, 5.0), (3, right_mass)] {
            world.data[[1, x]].cell_type = CellType::Wood;
            world.data[[2, x]].cell_type = CellType::Water;
            world.data[[2, x]].mass = mass;
        }
        let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(70.0, 70.0));
        let expected = if right_mass == 5.0 {
            vec![
                (
                    Rect::from_min_max(pos2(20.0, 15.0), pos2(40.0, 25.0)),
                    Color32::BROWN,
                ),
                (
                    Rect::from_min_max(pos2(20.0, 25.0), pos2(40.0, 30.0)),
                    Color32::BLUE,
                ),
            ]
        } else {
            vec![
                (
                    Rect::from_min_max(pos2(20.0, 15.0), pos2(30.0, 25.0)),
                    Color32::BROWN,
                ),
                (
                    Rect::from_min_max(pos2(30.0, 10.0), pos2(40.0, 20.0)),
                    Color32::BROWN,
                ),
                (
                    Rect::from_min_max(pos2(20.0, 25.0), pos2(30.0, 30.0)),
                    Color32::BLUE,
                ),
                (
                    Rect::from_min_max(pos2(30.0, 20.0), pos2(40.0, 30.0)),
                    Color32::BLUE,
                ),
            ]
        };
        assert_eq!(rectangles(&world, viewport), expected);
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

#[test]
fn rendering_seeded_surface_coverage_snapshot() {
    let mut hashes = Vec::new();
    for seed in 0..4 {
        for shift in [vec2(0.0, 0.0), vec2(-2.5, -7.5), vec2(5.0, 10.0)] {
            let mut world = fixtures::mixed_world(9, seed, true);
            world.use_shift = true;
            world.pos_shift = shift;
            let viewport = Rect::from_min_size(pos2(100.0, 200.0), vec2(35.0, 30.0));
            let shapes = rectangles(&world, viewport);
            let mut hash = 0xcbf2_9ce4_8422_2325u64;
            // Sample ordered opaque coverage at quarter-pixel spacing, including
            // fractional water/wood surfaces; this does not emulate GPU antialiasing.
            for y in 0..120 {
                for x in 0..140 {
                    let pos = viewport.min + vec2((x as f32 + 0.5) / 4.0, (y as f32 + 0.5) / 4.0);
                    let color = shapes
                        .iter()
                        .rev()
                        .find(|(rect, _)| {
                            pos.x >= rect.min.x
                                && pos.x < rect.max.x
                                && pos.y >= rect.min.y
                                && pos.y < rect.max.y
                        })
                        .map_or(Color32::WHITE, |(_, color)| *color);
                    for byte in color.to_array() {
                        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
                    }
                }
            }
            hashes.push(hash);
        }
    }
    eprintln!("Surface coverage snapshots: {hashes:?}");
    // Captured from the unmerged renderer at 563cfb2.
    assert_eq!(
        hashes,
        vec![
            814439034839001221,
            1019730208595234805,
            2796741114083359669,
            1798147957961862829,
            18395841835852489373,
            15747703100952878437,
            6956341954397828109,
            3293012435417082701,
            3750199894272108365,
            5262289592828001013,
            1883324261346571381,
            65812939947395317
        ]
    );
}
