use super::*;

#[test]
fn edit_state_smooth_preserves_live_materials_with_stale_scratch() {
    for size in [3, 4, 7] {
        for y in 1..size - 1 {
            for x in 1..size - 1 {
                for (material, mass) in [
                    (CellType::Water, 0.0),
                    (CellType::Water, MIN_MASS / 2.0),
                    (CellType::Water, MIN_MASS),
                    (CellType::Water, 20.0),
                    (CellType::Smoke, 1.25),
                    (CellType::DarkSmoke, 2.5),
                    (CellType::Sand, 0.0),
                    (CellType::Wood, 0.0),
                    (CellType::FireNormal, 0.0),
                    (CellType::FireBurn, 0.0),
                ] {
                    let mut world = World::new(size);
                    world.add_border();
                    world.data[[y, x]].cell_type = material;
                    world.data[[y, x]].mass = mass;
                    let original = world.data[[y, x]];
                    for cell in &mut world.tmp {
                        cell.mass = 99.0;
                    }
                    for _ in 0..3 {
                        world.smooth();
                        assert!(world.data[[y, x]] == original);
                        for cell in &world.data {
                            assert!(cell.mass.is_finite() && cell.mass >= 0.0);
                            if cell.x != x || cell.y != y {
                                assert_eq!(cell.mass, 0.0);
                            }
                            if cell.x == 0
                                || cell.y == 0
                                || cell.x == size - 1
                                || cell.y == size - 1
                            {
                                assert!(cell.cell_type == CellType::Dirt);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn edit_state_generation_resets_every_scalar_at_threshold_endpoints() {
    for size in [1, 2, 3, 4, 7] {
        for threshold in [0.0, 1.0] {
            let mut world = World::new(size);
            for ((y, x), cell) in world.data.indexed_iter_mut() {
                cell.cell_type = if (y + x) % 2 == 0 {
                    CellType::Water
                } else {
                    CellType::Smoke
                };
                cell.mass = (1 + y * size + x) as f32;
            }
            for cell in &mut world.tmp {
                cell.mass = 99.0;
            }
            world.threshold = threshold;
            world.rand_generate();
            for ((y, x), cell) in world.data.indexed_iter() {
                let border = y == 0 || x == 0 || y == size - 1 || x == size - 1;
                assert!(
                    cell.cell_type
                        == if border || threshold == 1.0 {
                            CellType::Dirt
                        } else {
                            CellType::None
                        }
                );
                assert_eq!(cell.mass, 0.0);
                assert_eq!((cell.y, cell.x), (y, x));
            }
            // Smoothing immediately after generation must not resurrect old mass.
            world.smooth();
            assert!(world.data.iter().all(|cell| cell.mass == 0.0));
        }
    }
}

#[test]
fn edit_state_clear_then_smooth_does_not_restore_previous_mass() {
    let mut world = World::new(7);
    world.data[[3, 3]].cell_type = CellType::Water;
    world.data[[3, 3]].mass = 7.0;
    world.tmp = world.data.clone();
    world.clear();
    world.smooth();
    assert!(world.data.iter().all(|cell| cell.mass == 0.0));
}

#[test]
fn edit_state_smooth_keeps_existing_cave_rule_for_all_neighbor_patterns() {
    let neighbors = [
        (2, 2),
        (2, 3),
        (2, 4),
        (3, 2),
        (3, 4),
        (4, 2),
        (4, 3),
        (4, 4),
    ];
    for mask in 0u16..256 {
        for original_type in [CellType::None, CellType::Dirt] {
            let mut world = World::new(7);
            world.add_border();
            world.data[[3, 3]].cell_type = original_type;
            for (bit, (y, x)) in neighbors.into_iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    world.data[[y, x]].cell_type = CellType::Dirt;
                }
            }
            world.smooth();
            let expected = match mask.count_ones() {
                0..=3 => CellType::None,
                4 => original_type,
                _ => CellType::Dirt,
            };
            assert!(world.data[[3, 3]].cell_type == expected, "mask={mask}");
        }
    }
}
