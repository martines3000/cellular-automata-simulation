use super::*;

fn solid_world(size: usize) -> World {
    let mut world = World::new(size);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    world.rng = StdRng::seed_from_u64(42);
    world
}

fn put(world: &mut World, at: [usize; 2], kind: CellType, mass: f32) {
    world.data[at] = Cell {
        cell_type: kind,
        mass,
    };
}

fn quantity(world: &World) -> f64 {
    world
        .data
        .iter()
        .map(|cell| f64::from(cell.water_quantity()))
        .sum()
}

fn phase(world: &mut World) {
    let before = quantity(world);
    let border: Vec<_> = world
        .data
        .indexed_iter()
        .filter(|((y, x), _)| {
            *y == 0 || *x == 0 || *y + 1 == world.data.nrows() || *x + 1 == world.data.ncols()
        })
        .map(|(index, cell)| (index, *cell))
        .collect();
    world.tmp.assign(&world.data);
    world.update_frozen();
    assert_eq!(
        quantity(world),
        before,
        "frozen transport does no arithmetic on quantity"
    );
    for (at, original) in border {
        assert!(world.data[at] == original);
    }
    assert!(world
        .data
        .iter()
        .all(|cell| cell.mass.is_finite() && cell.mass >= 0.0));
}

fn brush(world: &mut World, at: [usize; 2], tool: PhaseTool) {
    let clip = Rect::from_min_size(pos2(50.0, 100.0), vec2(500.0, 500.0));
    let pos = clip.min
        + world.pos_shift
        + vec2(
            (at[1] as f32 + 0.5) * world.block_size,
            (at[0] as f32 + 0.5) * world.block_size,
        );
    world.apply_phase_brush(Some(pos), clip, tool);
}

#[test]
fn frozen_quantity_excludes_smoke_lifetime_and_other_material_scalars() {
    for kind in [
        CellType::None,
        CellType::Water,
        CellType::Ice,
        CellType::Snow,
    ] {
        assert_eq!(
            (Cell {
                cell_type: kind,
                mass: 25.0
            })
            .water_quantity(),
            25.0
        );
    }
    for kind in [
        CellType::Smoke,
        CellType::DarkSmoke,
        CellType::Wood,
        CellType::Dirt,
        CellType::Sand,
        CellType::FireNormal,
        CellType::FireBurn,
    ] {
        assert_eq!(
            (Cell {
                cell_type: kind,
                mass: 25.0
            })
            .water_quantity(),
            0.0
        );
    }
}

#[test]
fn frozen_brush_roundtrip_preserves_exact_quantity_at_thresholds_and_high_pressure() {
    for kind in [CellType::None, CellType::Water] {
        for mass in [
            0.0,
            0.005,
            f32::from_bits(MIN_MASS.to_bits() - 1),
            MIN_MASS,
            f32::from_bits(MIN_MASS.to_bits() + 1),
            1.0,
            10.0,
            25.0,
            f32::MAX,
        ] {
            let mut world = solid_world(7);
            put(&mut world, [3, 3], kind, mass);
            world.block_size = 10.0;
            world.shift = vec2(2.0, 1.0);
            world.update_pos();
            let original = world.data.clone();
            for _ in 0..10 {
                brush(&mut world, [3, 3], PhaseTool::Freeze);
                assert_eq!(world.data[[3, 3]].mass.to_bits(), mass.to_bits());
                assert!(
                    world.data[[3, 3]].cell_type == if mass > 0.0 { CellType::Ice } else { kind }
                );
                brush(&mut world, [3, 3], PhaseTool::Melt);
                assert_eq!(world.data[[3, 3]].mass.to_bits(), mass.to_bits());
                if mass > 0.0 {
                    assert!(
                        world.data[[3, 3]].cell_type
                            == if mass >= MIN_MASS {
                                CellType::Water
                            } else {
                                CellType::None
                            }
                    );
                }
                for ((y, x), cell) in world.data.indexed_iter() {
                    if [y, x] != [3, 3] {
                        assert!(*cell == original[[y, x]]);
                    }
                }
            }
        }
    }
}

#[test]
fn frozen_brush_protects_borders_unrelated_materials_and_off_canvas_positions() {
    for kind in [
        CellType::Dirt,
        CellType::Sand,
        CellType::Wood,
        CellType::FireNormal,
        CellType::Smoke,
    ] {
        let mut world = solid_world(7);
        put(&mut world, [3, 3], kind, 1.0);
        let original = world.data.clone();
        for tool in [PhaseTool::Freeze, PhaseTool::Melt] {
            brush(&mut world, [3, 3], tool);
            brush(&mut world, [0, 3], tool);
            brush(&mut world, [6, 3], tool);
            let clip = Rect::from_min_size(Pos2::ZERO, vec2(100.0, 100.0));
            for pos in [
                None,
                Some(pos2(-1.0, 10.0)),
                Some(pos2(100.0, 10.0)),
                Some(pos2(10.0, 100.0)),
                Some(pos2(f32::NAN, 10.0)),
            ] {
                world.apply_phase_brush(pos, clip, tool);
            }
            assert!(world.data == original);
        }
    }
}

#[test]
fn frozen_painting_has_explicit_water_sources_and_eraser_sink() {
    for (kind, amount) in [(CellType::Ice, MAX_MASS), (CellType::Snow, 1.0)] {
        let mut world = World::new(7);
        world.add_border();
        let clip = Rect::from_min_size(Pos2::ZERO, vec2(100.0, 100.0));
        let pos = Some(pos2(17.5, 17.5));
        world.selected_cell_type = kind;
        world.transform_cell(pos, clip);
        world.transform_cell(pos, clip);
        assert_eq!(quantity(&world), f64::from(amount)); // Replacement, not additive painting.
        brush(&mut world, [3, 3], PhaseTool::Melt);
        assert_eq!(quantity(&world), f64::from(amount));
        world.selected_cell_type = CellType::None;
        world.transform_cell(pos, clip);
        assert_eq!(quantity(&world), 0.0);
    }
}

#[test]
fn frozen_falling_ice_and_snow_preserve_passive_water_and_move_one_row() {
    for kind in [CellType::Ice, CellType::Snow] {
        for residue in [0.0, 0.005, MIN_MASS] {
            let mut world = solid_world(7);
            put(&mut world, [2, 3], kind, 1.25);
            for y in 3..6 {
                put(&mut world, [y, 3], CellType::None, 0.0);
            }
            world.data[[3, 3]].mass = residue;
            phase(&mut world);
            assert!(world.data[[3, 3]].cell_type == kind);
            assert_eq!(world.data[[3, 3]].mass, 1.25);
            assert_eq!(world.data[[2, 3]].mass, residue);
            assert!(world.data[[4, 3]].cell_type == CellType::None);
        }
    }
}

#[test]
fn frozen_ice_rises_one_row_without_losing_destination_water() {
    for water in [MIN_MASS, 1.0, 25.0] {
        let mut world = solid_world(7);
        put(&mut world, [4, 3], CellType::Ice, 10.0);
        put(&mut world, [3, 3], CellType::Water, water);
        put(&mut world, [2, 3], CellType::Water, 10.0);
        phase(&mut world);
        assert!(world.data[[3, 3]].cell_type == CellType::Ice);
        assert_eq!(world.data[[4, 3]].mass, water);
        assert!(world.data[[4, 3]].cell_type == CellType::Water);
        assert!(world.data[[2, 3]].cell_type == CellType::Water);
    }
}

#[test]
fn frozen_ice_rests_at_surface_and_stacks_do_not_overwrite_each_other() {
    let mut world = solid_world(7);
    put(&mut world, [2, 3], CellType::None, 0.005);
    put(&mut world, [3, 3], CellType::Ice, 10.0);
    put(&mut world, [4, 3], CellType::Water, 10.0);
    let original = world.data.clone();
    phase(&mut world);
    assert!(world.data == original);
    put(&mut world, [2, 3], CellType::Ice, 2.0);
    put(&mut world, [4, 3], CellType::Ice, 4.0);
    put(&mut world, [5, 3], CellType::None, 0.005);
    phase(&mut world);
    assert_eq!(world.data[[2, 3]].mass, 2.0);
    assert_eq!(world.data[[3, 3]].mass, 10.0);
    assert_eq!(world.data[[5, 3]].mass, 4.0);
}

#[test]
fn frozen_snow_slides_only_through_open_sides_and_resolves_shared_destinations() {
    for direction in [-1, 1] {
        let nx = (3 + direction) as usize;
        let mut world = solid_world(7);
        put(&mut world, [2, 3], CellType::Snow, 1.0);
        put(&mut world, [3, nx], CellType::None, 0.005);
        phase(&mut world);
        assert!(world.data[[2, 3]].cell_type == CellType::Snow); // Corner is blocked.
        put(&mut world, [2, nx], CellType::None, 0.0);
        phase(&mut world);
        assert!(world.data[[3, nx]].cell_type == CellType::Snow);
        assert_eq!(world.data[[2, 3]].mass, 0.005);
    }
    let mut world = solid_world(7);
    put(&mut world, [2, 2], CellType::Snow, 1.0);
    put(&mut world, [2, 4], CellType::Snow, 2.0);
    put(&mut world, [2, 3], CellType::None, 0.0);
    put(&mut world, [3, 3], CellType::None, 0.005);
    phase(&mut world);
    assert!(world.data[[3, 3]].cell_type == CellType::Snow);
    assert_eq!(
        world
            .data
            .iter()
            .filter(|cell| cell.cell_type == CellType::Snow)
            .count(),
        2
    );
}

#[test]
fn frozen_snow_water_contact_is_cardinal_and_preserves_small_residues() {
    for mass in [0.005, MIN_MASS, 1.0] {
        for (dy, dx) in SMALL_NEIGHBOURHOOD {
            let mut world = solid_world(7);
            put(&mut world, [3, 3], CellType::Snow, mass);
            put(
                &mut world,
                [(3 + dy) as usize, (3 + dx) as usize],
                CellType::Water,
                25.0,
            );
            phase(&mut world);
            assert!(
                world.data[[3, 3]].cell_type
                    == if mass >= MIN_MASS {
                        CellType::Water
                    } else {
                        CellType::None
                    }
            );
            assert_eq!(world.data[[3, 3]].mass, mass);
        }
    }
    let mut world = solid_world(7);
    put(&mut world, [3, 3], CellType::Snow, 1.0);
    put(&mut world, [2, 2], CellType::Water, 10.0);
    phase(&mut world);
    assert!(world.data[[3, 3]].cell_type == CellType::Snow);
}

#[test]
fn frozen_fire_melts_each_cardinal_target_once_without_touching_other_solids() {
    for kind in [CellType::Ice, CellType::Snow] {
        for fire in [CellType::FireNormal, CellType::FireBurn] {
            for mass in [0.005, MIN_MASS, 1.0, 25.0] {
                let mut world = solid_world(7);
                put(&mut world, [3, 3], kind, mass);
                for (dy, dx) in SMALL_NEIGHBOURHOOD {
                    put(
                        &mut world,
                        [(3 + dy) as usize, (3 + dx) as usize],
                        fire,
                        0.0,
                    );
                }
                let before = quantity(&world);
                world.tmp.assign(&world.data);
                world.update_fire();
                assert_eq!(quantity(&world), before);
                assert_eq!(world.data[[3, 3]].mass, mass);
                assert!(
                    world.data[[3, 3]].cell_type
                        == if mass >= MIN_MASS {
                            CellType::Water
                        } else {
                            CellType::None
                        }
                );
                assert!(world.data[[0, 3]].cell_type == CellType::Dirt);
            }
        }
    }
}

#[test]
fn frozen_minimal_worlds_and_edits_preserve_borders_and_material_state() {
    for size in [3, 4, 7] {
        for kind in [CellType::Ice, CellType::Snow] {
            let mut world = solid_world(size);
            put(&mut world, [1, 1], kind, 1.0);
            phase(&mut world);
            world.smooth();
            assert!(world.data[[1, 1]].cell_type == kind);
            assert_eq!(world.data[[1, 1]].mass, 1.0);
            world.clear();
            assert_eq!(quantity(&world), 0.0);
            world.resize_and_clear(size);
            assert!(world.data == world.tmp);
        }
    }
}

#[test]
fn frozen_seeded_mixed_ticks_budget_all_phases_and_match_the_schedule() {
    for size in [5, 7, 16] {
        for seed in 0..32 {
            let fixture = || {
                let mut world = fixtures::mixed_world(size, seed, true);
                let mut rng = StdRng::seed_from_u64(seed ^ 0x189b);
                for y in 1..size - 1 {
                    for x in 1..size - 1 {
                        if rng.gen_bool(0.25) {
                            let kind = if rng.gen_bool(0.5) {
                                CellType::Ice
                            } else {
                                CellType::Snow
                            };
                            put(
                                &mut world,
                                [y, x],
                                kind,
                                if rng.gen_bool(0.25) {
                                    0.005
                                } else if kind == CellType::Ice {
                                    10.0
                                } else {
                                    1.0
                                },
                            );
                        }
                    }
                }
                world
            };
            let mut world = fixture();
            let mut reference = fixture();
            for tick in 0..24 {
                mixed_tests::checked_tick(
                    &mut reference,
                    &format!("frozen size={size} seed={seed} tick={tick}"),
                );
                world.tick();
                assert!(world.data == reference.data);
                assert!(world.tmp == reference.tmp);
            }
        }
    }
}

#[test]
fn frozen_rendering_uses_distinct_colors_and_merges_adjacent_equal_materials() {
    let mut world = World::new(7);
    put(&mut world, [2, 2], CellType::Ice, 10.0);
    put(&mut world, [2, 3], CellType::Ice, 1.0);
    put(&mut world, [2, 4], CellType::Snow, 1.0);
    let mut shapes = Vec::new();
    world.gen_shapes(
        &mut shapes,
        Rect::from_min_size(Pos2::ZERO, vec2(100.0, 100.0)),
    );
    assert_eq!(shapes.len(), 2);
    assert!(
        matches!(&shapes[0], Shape::Rect(rect) if rect.fill == ICE_COLOR && rect.rect.width() == 10.0)
    );
    assert!(matches!(&shapes[1], Shape::Rect(rect) if rect.fill == SNOW_COLOR));
}

#[test]
fn frozen_large_world_full_tick_keeps_quantity_budgets_and_solid_borders() {
    for size in [100, 1024] {
        let mut world = fixtures::mixed_world(size, 91, true);
        for y in (1..size - 1).step_by(17) {
            for x in (1..size - 1).step_by(19) {
                let kind = if (x + y) % 2 == 0 {
                    CellType::Ice
                } else {
                    CellType::Snow
                };
                put(&mut world, [y, x], kind, 1.0);
            }
        }
        mixed_tests::checked_tick(&mut world, &format!("frozen large size={size}"));
    }
}
