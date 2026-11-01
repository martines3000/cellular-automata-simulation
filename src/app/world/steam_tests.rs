use super::*;

fn solid(size: usize) -> World {
    let mut world = World::new(size);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    world.rng = StdRng::seed_from_u64(8);
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
    let solids: Vec<_> = world
        .data
        .indexed_iter()
        .filter(|(_, cell)| cell.cell_type == CellType::Dirt)
        .map(|(at, cell)| (at, *cell))
        .collect();
    world.tmp.assign(&world.data);
    world.update_steam();
    assert_eq!(
        quantity(world),
        before,
        "gas transport and condensation do no quantity arithmetic"
    );
    for (at, cell) in solids {
        assert!(world.data[at] == cell);
    }
    assert!(world
        .data
        .iter()
        .all(|cell| cell.mass.is_finite() && cell.mass >= 0.0));
}

fn brush(world: &mut World, tool: PhaseTool) {
    let rect = Rect::from_min_size(pos2(50.0, 100.0), vec2(500.0, 500.0));
    let pos = rect.min + world.pos_shift + Vec2::splat(3.5 * world.block_size);
    world.apply_phase_brush(Some(pos), rect, tool);
}

#[test]
fn steam_phase_brushes_preserve_every_quantity_bit_and_ignore_other_materials() {
    let masses = [
        0.0,
        0.005,
        f32::from_bits(MIN_MASS.to_bits() - 1),
        MIN_MASS,
        f32::from_bits(MIN_MASS.to_bits() + 1),
        1.0,
        25.0,
        f32::MAX,
    ];
    for kind in [CellType::Water, CellType::None] {
        for mass in masses {
            let mut world = solid(7);
            world.block_size = 10.0;
            world.shift = vec2(1.0, 2.0);
            world.update_pos();
            put(&mut world, [3, 3], kind, mass);
            for _ in 0..10 {
                brush(&mut world, PhaseTool::Vaporize);
                assert!(
                    world.data[[3, 3]].cell_type == if mass > 0.0 { CellType::Steam } else { kind }
                );
                assert_eq!(world.data[[3, 3]].mass.to_bits(), mass.to_bits());
                brush(&mut world, PhaseTool::Condense);
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
            }
        }
    }
    for kind in [
        CellType::Dirt,
        CellType::Sand,
        CellType::Wood,
        CellType::Smoke,
        CellType::DarkSmoke,
        CellType::FireNormal,
        CellType::Ice,
        CellType::Snow,
    ] {
        let mut world = solid(7);
        put(&mut world, [3, 3], kind, 1.0);
        let original = world.data.clone();
        brush(&mut world, PhaseTool::Vaporize);
        brush(&mut world, PhaseTool::Condense);
        assert!(world.data == original);
    }
}

#[test]
fn steam_painting_is_an_explicit_source_and_erasing_is_a_sink() {
    let mut world = World::new(7);
    world.add_border();
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(100.0, 100.0));
    let pos = Some(pos2(17.5, 17.5));
    world.selected_cell_type = CellType::Steam;
    world.transform_cell(pos, rect);
    world.transform_cell(pos, rect);
    assert_eq!(quantity(&world), 1.0);
    brush(&mut world, PhaseTool::Condense);
    assert_eq!(quantity(&world), 1.0);
    world.selected_cell_type = CellType::None;
    world.transform_cell(pos, rect);
    assert_eq!(quantity(&world), 0.0);
}

#[test]
fn steam_rises_one_cell_and_preserves_liquid_or_passive_water() {
    for kind in [CellType::None, CellType::Water] {
        for residue in [0.0, 0.005, MIN_MASS] {
            let mut world = solid(7);
            put(&mut world, [4, 3], CellType::Steam, 1.25);
            put(&mut world, [3, 3], kind, residue);
            put(&mut world, [2, 3], CellType::None, 0.0);
            phase(&mut world);
            assert!(world.data[[3, 3]].cell_type == CellType::Steam);
            assert_eq!(world.data[[3, 3]].mass, 1.25);
            assert_eq!(world.data[[4, 3]].mass, residue);
            assert!(world.data[[2, 3]].cell_type == CellType::None);
        }
    }
    let mut world = solid(7);
    put(&mut world, [3, 3], CellType::Steam, 0.005);
    put(&mut world, [2, 3], CellType::Water, 25.0);
    phase(&mut world);
    assert_eq!(world.data[[3, 3]].mass, 25.0);
    assert!(world.data[[3, 3]].cell_type == CellType::Water);
    assert_eq!(world.data[[2, 3]].mass, 0.005);
}

#[test]
fn steam_diagonal_and_sideways_motion_preserve_residue_and_respect_corners() {
    for direction in [-1, 1] {
        let nx = (3 + direction) as usize;
        for ny in [2, 3] {
            let mut world = solid(7);
            put(&mut world, [3, 3], CellType::Steam, 1.0);
            put(&mut world, [ny, nx], CellType::None, 0.005);
            if ny == 2 {
                // A blocked side prevents crossing the solid corner.
                phase(&mut world);
                assert!(world.data[[3, 3]].cell_type == CellType::Steam);
                put(&mut world, [3, nx], CellType::None, 0.0);
            }
            phase(&mut world);
            assert!(world.data[[ny, nx]].cell_type == CellType::Steam);
            assert_eq!(world.data[[3, 3]].mass, 0.005);
        }
    }
}

#[test]
fn steam_competing_destinations_and_old_sources_do_not_overwrite_parcels() {
    let mut world = solid(7);
    put(&mut world, [3, 2], CellType::Steam, 1.0);
    put(&mut world, [3, 4], CellType::Steam, 2.0);
    put(&mut world, [3, 3], CellType::None, 0.005);
    put(&mut world, [2, 3], CellType::None, 0.005);
    phase(&mut world);
    assert_eq!(
        world
            .data
            .iter()
            .filter(|cell| cell.cell_type == CellType::Steam)
            .count(),
        2
    );
    let mut world = solid(7);
    put(&mut world, [3, 3], CellType::Steam, 2.0);
    put(&mut world, [4, 3], CellType::Steam, 1.0);
    put(&mut world, [2, 3], CellType::None, 0.0);
    phase(&mut world);
    assert_eq!(world.data[[2, 3]].mass, 2.0);
    assert_eq!(world.data[[4, 3]].mass, 1.0);
}

#[test]
fn steam_in_closed_minimal_worlds_never_expires_or_erodes_borders() {
    for size in [3, 4, 7] {
        for mass in [0.005, MIN_MASS, 1.0, 25.0] {
            let mut world = solid(size);
            put(&mut world, [1, 1], CellType::Steam, mass);
            let original = world.data.clone();
            for _ in 0..512 {
                world.tick(); // Include smoke lifetime processing in the production schedule.
                assert!(world.data == original);
            }
            assert!(world.data == original);
        }
    }
    let mut world = solid(3);
    put(&mut world, [1, 1], CellType::Steam, 0.0);
    phase(&mut world);
    assert!(world.data[[1, 1]].cell_type == CellType::None);
}

#[test]
fn steam_condenses_at_cold_cardinal_contacts_without_changing_the_solid() {
    for kind in [CellType::Ice, CellType::Snow] {
        for mass in [0.005, MIN_MASS, 25.0] {
            for (dy, dx) in SMALL_NEIGHBOURHOOD {
                let mut world = solid(7);
                let cold = [(3 + dy) as usize, (3 + dx) as usize];
                put(&mut world, [3, 3], CellType::Steam, mass);
                put(&mut world, cold, kind, 1.0);
                let original = world.data[cold];
                phase(&mut world);
                assert!(world.data[cold] == original);
                assert_eq!(world.data[[3, 3]].mass, mass);
                assert!(
                    world.data[[3, 3]].cell_type
                        == if mass >= MIN_MASS {
                            CellType::Water
                        } else {
                            CellType::None
                        }
                );
            }
        }
    }
    let mut world = solid(7);
    put(&mut world, [3, 3], CellType::Steam, 1.0);
    put(&mut world, [2, 2], CellType::Ice, 10.0);
    phase(&mut world);
    assert!(world.data[[3, 3]].cell_type == CellType::Steam);
}

#[test]
fn steam_fire_vaporizes_water_once_and_does_not_chain_react_new_meltwater() {
    for kind in [CellType::Water, CellType::Ice, CellType::Snow] {
        for mass in [0.005, MIN_MASS, 1.0, 25.0] {
            let mut world = solid(7);
            put(&mut world, [3, 3], kind, mass);
            for (dy, dx) in SMALL_NEIGHBOURHOOD {
                put(
                    &mut world,
                    [(3 + dy) as usize, (3 + dx) as usize],
                    CellType::FireNormal,
                    0.0,
                );
            }
            let before = quantity(&world);
            world.tmp.assign(&world.data);
            world.update_fire();
            assert_eq!(quantity(&world), before);
            assert_eq!(world.data[[3, 3]].mass, mass);
            let expected = if kind == CellType::Water {
                CellType::Steam
            } else if mass >= MIN_MASS {
                CellType::Water
            } else {
                CellType::None
            };
            assert!(world.data[[3, 3]].cell_type == expected);
        }
    }
}

#[test]
fn steam_seeded_mixed_ticks_preserve_every_phase_budget_and_match_production() {
    for size in [5, 7, 16] {
        for seed in 0..32 {
            let fixture = || {
                let mut world = fixtures::mixed_world(size, seed, true);
                let mut rng = StdRng::seed_from_u64(seed ^ 0x1ae4);
                for y in 1..size - 1 {
                    for x in 1..size - 1 {
                        let kind = match rng.gen_range(0..8) {
                            0 => CellType::Steam,
                            1 => CellType::Ice,
                            2 => CellType::Snow,
                            _ => continue,
                        };
                        let masses = [0.005, MIN_MASS, 1.0, 10.0, 25.0];
                        put(
                            &mut world,
                            [y, x],
                            kind,
                            masses[rng.gen_range(0..masses.len())],
                        );
                    }
                }
                world
            };
            let mut world = fixture();
            let mut reference = fixture();
            for tick in 0..24 {
                mixed_tests::checked_tick(
                    &mut reference,
                    &format!("steam size={size} seed={seed} tick={tick}"),
                );
                world.tick();
                assert!(world.data == reference.data);
                assert!(world.tmp == reference.tmp);
            }
        }
    }
}

#[test]
fn steam_large_world_mixed_tick_preserves_material_budgets() {
    for size in [100, 1024] {
        let mut world = fixtures::mixed_world(size, 45, true);
        for y in (1..size - 1).step_by(17) {
            for x in (1..size - 1).step_by(19) {
                put(&mut world, [y, x], CellType::Steam, 1.0);
            }
        }
        mixed_tests::checked_tick(&mut world, &format!("steam large size={size}"));
    }
}

#[test]
fn steam_rendering_has_its_own_color_and_edits_reset_quantity() {
    let mut world = solid(7);
    put(&mut world, [3, 3], CellType::Steam, 25.0);
    world.smooth();
    assert!(world.data[[3, 3]].cell_type == CellType::Steam);
    assert_eq!(quantity(&world), 25.0);
    assert_eq!(world.data[[3, 3]].color(), STEAM_COLOR);
    assert_ne!(
        STEAM_COLOR,
        Cell {
            cell_type: CellType::Smoke,
            mass: 1.0
        }
        .color()
    );
    world.clear();
    assert_eq!(quantity(&world), 0.0);
    world.resize_and_clear(100);
    assert!(world.data == world.tmp);
    assert_eq!(quantity(&world), 0.0);
}
