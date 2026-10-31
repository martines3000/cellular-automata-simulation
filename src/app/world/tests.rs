use super::*;

fn bordered_world() -> World {
    let mut world = World::new(7);
    world.add_border();
    // Ineligible materials must survive a water-only phase, including their scalars.
    world.data[[1, 1]].cell_type = CellType::Sand;
    world.data[[1, 5]].cell_type = CellType::Smoke;
    world.data[[1, 5]].mass = 3.0;
    world
}

#[test]
fn painting_preserves_border_and_rejects_off_canvas_positions() {
    let mut world = World::new(20);
    world.add_border();
    let rect = Rect::from_min_size(pos2(100.0, 200.0), vec2(20.0, 20.0));
    let before = world.data.clone();
    for pos in [
        None,
        Some(rect.min),
        Some(rect.max),
        Some(rect.max + vec2(5.0, 5.0)),
        Some(rect.min - vec2(1.0, 1.0)),
    ] {
        world.transform_cell(pos, rect);
        assert!(world.data == before);
    }
}

fn water(world: &mut World, y: usize, x: usize, mass: f32) {
    world.data[[y, x]].cell_type = CellType::Water;
    world.data[[y, x]].mass = mass;
}

fn fluid_mass(world: &World) -> f64 {
    // None can hold incoming quantities below the active-water threshold.
    world
        .data
        .iter()
        .filter(|cell| matches!(cell.cell_type, CellType::Water | CellType::None))
        .map(|cell| f64::from(cell.mass))
        .sum()
}

fn phase(world: &mut World) {
    let protected: Vec<Cell> = world
        .data
        .iter()
        .filter(|cell| !matches!(cell.cell_type, CellType::Water | CellType::None))
        .copied()
        .collect();
    world.tmp = world.data.clone();
    world.update_water();
    for cell in protected {
        assert!(world.data[[cell.y, cell.x]] == cell);
    }
    for cell in &world.data {
        assert!(cell.mass.is_finite() && cell.mass >= 0.0);
        if cell.cell_type == CellType::None {
            assert!(cell.mass <= MIN_MASS);
        }
        // An active source may drain below MIN_MASS and remain Water until the
        // next phase's cleanup; do not impose a new global normalization rule.
    }
    let edge = world.data.dim().0 - 1;
    for i in 0..=edge {
        for (y, x) in [(0, i), (edge, i), (i, 0), (i, edge)] {
            assert!(world.data[[y, x]].cell_type == CellType::Dirt);
            assert_eq!(world.data[[y, x]].mass, 0.0);
        }
    }
}

fn assert_budget(world: &World, before: f64, discarded: f64) {
    let expected = before - discarded;
    // Bound f32 transfer rounding well below any deliberate cleanup residue.
    let rounding = 4.0 * f64::from(f32::EPSILON) * before.max(1.0);
    assert!(
        (fluid_mass(world) - expected).abs() <= rounding,
        "mass {} != {before} - {discarded}",
        fluid_mass(world)
    );
}

#[test]
fn water_cleanup_preserves_one_unit_over_tiny_source() {
    let mut world = bordered_world();
    water(&mut world, 2, 3, 1.0);
    water(&mut world, 3, 3, 0.005);
    phase(&mut world);
    assert_eq!(fluid_mass(&world), 1.0); // Only the old 0.005 residue is discarded.
    assert_eq!(world.data[[3, 3]].mass, 1.0);
    assert!(world.data[[3, 3]].cell_type == CellType::Water);
}

#[test]
fn water_cleanup_preserves_incoming_into_zero_source() {
    let mut world = bordered_world();
    water(&mut world, 2, 3, 1.0);
    water(&mut world, 3, 3, 0.0);
    phase(&mut world);
    assert_eq!(fluid_mass(&world), 1.0);
    assert_eq!(world.data[[3, 3]].mass, 1.0);
    assert!(world.data[[3, 3]].cell_type == CellType::Water);
}

#[test]
fn water_cleanup_old_source_threshold_with_incoming() {
    let below = f32::from_bits(MIN_MASS.to_bits() - 1);
    let above = f32::from_bits(MIN_MASS.to_bits() + 1);
    for old_mass in [below, MIN_MASS, above] {
        let mut world = bordered_world();
        world.data[[4, 3]].cell_type = CellType::Dirt;
        water(&mut world, 2, 3, 1.0);
        water(&mut world, 3, 3, old_mass);
        let discarded = if old_mass < MIN_MASS { old_mass } else { 0.0 };
        let before = fluid_mass(&world);
        phase(&mut world);
        assert_budget(&world, before, f64::from(discarded));
        assert_eq!(world.data[[3, 3]].mass, (1.0 + old_mass) - discarded);
        assert!(world.data[[3, 3]].cell_type == CellType::Water);
    }
}

#[test]
fn water_cleanup_isolated_tiny_source_is_discarded() {
    let mut world = bordered_world();
    water(&mut world, 3, 3, 0.005);
    phase(&mut world);
    assert_eq!(fluid_mass(&world), 0.0); // Existing isolated-residue sink is retained.
    assert_eq!(world.data[[3, 3]].mass, 0.0);
    assert!(world.data[[3, 3]].cell_type == CellType::None);
}

#[test]
fn water_cleanup_retained_incoming_threshold() {
    // Isolate cleanup of an already credited accumulator, including passive
    // incoming quantities smaller than normal flows in the fixtures above.
    for incoming in [
        f32::from_bits(MIN_MASS.to_bits() - 1),
        MIN_MASS,
        f32::from_bits(MIN_MASS.to_bits() + 1),
    ] {
        let mut world = bordered_world();
        water(&mut world, 3, 3, 0.0);
        world.tmp = world.data.clone();
        world.tmp[[3, 3]].mass += incoming;
        world.update_water();
        assert_eq!(fluid_mass(&world), f64::from(incoming));
        assert_eq!(world.data[[3, 3]].mass, incoming);
        assert!(
            world.data[[3, 3]].cell_type
                == if incoming >= MIN_MASS {
                    CellType::Water
                } else {
                    CellType::None
                }
        );
    }
}

fn supported_basin() -> World {
    let mut world = bordered_world();
    for y in 2..6 {
        for x in 1..6 {
            world.data[[y, x]].cell_type = CellType::Dirt;
        }
    }
    water(&mut world, 2, 3, 1.0);
    water(&mut world, 3, 3, 4.0);
    world.data[[3, 2]].cell_type = CellType::None;
    water(&mut world, 3, 4, 1.0 / 256.0);
    world
}

#[test]
fn water_cleanup_simultaneous_incoming_outgoing_budget() {
    let mut world = supported_basin();
    let before = fluid_mass(&world);
    phase(&mut world);
    assert_budget(&world, before, 1.0 / 256.0);
    assert_eq!(fluid_mass(&world), 5.0);
    assert_eq!(world.data[[3, 4]].mass, 1.0 - 1.0 / 1024.0);
    assert!(world.data[[3, 4]].cell_type == CellType::Water);
    assert!(world.data[[3, 3]].mass > 1.0 && world.data[[3, 3]].mass < 4.0);
    assert!(world.data[[3, 2]].mass > MIN_MASS);
}

#[test]
fn water_cleanup_repeated_closed_phases() {
    let mut world = supported_basin();
    let before = fluid_mass(&world);
    for _ in 0..16 {
        phase(&mut world);
        assert_budget(&world, before, 1.0 / 256.0);
        for x in 2..5 {
            assert!(world.data[[3, x]].mass >= MIN_MASS);
            assert!(world.data[[3, x]].cell_type == CellType::Water);
        }
    }
}

#[test]
fn water_cleanup_closed_mass_matrix_accounts_for_every_residue() {
    let masses = [
        0.0,
        MIN_MASS / 2.0,
        f32::from_bits(MIN_MASS.to_bits() - 1),
        MIN_MASS,
        f32::from_bits(MIN_MASS.to_bits() + 1),
        MIN_FLOW,
        1.0,
        MAX_MASS,
        MAX_MASS * 2.0,
    ];
    for mut configuration in 0..masses.len().pow(4) {
        let mut world = World::new(4);
        world.add_border();
        for (y, x) in [(1, 1), (1, 2), (2, 1), (2, 2)] {
            water(&mut world, y, x, masses[configuration % masses.len()]);
            configuration /= masses.len();
        }
        for _ in 0..5 {
            let before = fluid_mass(&world);
            let discarded: f64 = world
                .data
                .iter()
                .filter(|cell| cell.cell_type == CellType::Water && cell.mass < MIN_MASS)
                .map(|cell| f64::from(cell.mass))
                .sum();
            phase(&mut world);
            assert_budget(&world, before, discarded);
        }
    }
}

fn assert_supported_fire(size: usize, y: usize, x: usize, fire_type: CellType, wood_mask: u8) {
    let mut world = World::new(size);
    world.add_border();
    for dy in -1..=1 {
        for dx in -1..=1 {
            let ny = (y as i32 + dy) as usize;
            let nx = (x as i32 + dx) as usize;
            if (1..size - 1).contains(&ny) && (1..size - 1).contains(&nx) {
                world.data[[ny, nx]].cell_type = CellType::Wood;
            }
        }
    }
    for (bit, (ny, nx)) in [(y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)]
        .into_iter()
        .enumerate()
    {
        if world.data[[ny, nx]].cell_type == CellType::Wood && wood_mask & (1 << bit) == 0 {
            world.data[[ny, nx]].cell_type = CellType::Dirt;
        }
    }
    world.data[[y, x]].cell_type = fire_type;
    let mut expected = world.data.clone();
    expected[[y, x]].cell_type = if fire_type == CellType::FireBurn {
        CellType::DarkSmoke
    } else {
        CellType::Smoke
    };
    expected[[y, x]].mass = if fire_type == CellType::FireBurn {
        2.0
    } else {
        1.0
    };
    for ((ny, nx), cell) in expected.indexed_iter_mut() {
        if ny.abs_diff(y) + nx.abs_diff(x) == 1 && cell.cell_type == CellType::Wood {
            cell.cell_type = CellType::FireBurn;
        }
    }
    world.tmp = world.data.clone();
    world.update_fire();
    assert!(
        world.data == expected,
        "size={size}, y={y}, x={x}, mask={wood_mask}"
    );
}

#[test]
fn fire_indexing_supported_fire_ignites_only_cardinal_wood() {
    // Every interior position in these sizes, both fire types, and all 16
    // wood/solid cardinal patterns. Diagonal wood and borders stay unchanged.
    for size in [3, 4, 7] {
        for y in 1..size - 1 {
            for x in 1..size - 1 {
                for fire_type in [CellType::FireNormal, CellType::FireBurn] {
                    for wood_mask in 0..16 {
                        assert_supported_fire(size, y, x, fire_type, wood_mask);
                    }
                }
            }
        }
    }
}

#[test]
fn fire_indexing_nonwood_neighbors_are_unchanged() {
    for material in [
        CellType::Water,
        CellType::Sand,
        CellType::Smoke,
        CellType::DarkSmoke,
    ] {
        let mut world = World::new(7);
        world.add_border();
        for (y, x) in [(2, 3), (4, 3), (3, 2), (3, 4)] {
            world.data[[y, x]].cell_type = material;
            world.data[[y, x]].mass = 3.0;
        }
        world.data[[3, 3]].cell_type = CellType::FireNormal;
        let mut expected = world.data.clone();
        expected[[3, 3]].cell_type = CellType::Smoke;
        expected[[3, 3]].mass = 1.0;
        world.tmp = world.data.clone();
        world.update_fire();
        assert!(world.data == expected);
    }
}

#[test]
fn fire_indexing_unsupported_fire_keeps_existing_movement() {
    let mut world = World::new(7);
    world.add_border();
    world.data[[2, 3]].cell_type = CellType::FireBurn;
    world.data[[2, 2]].cell_type = CellType::Wood;
    let mut expected = world.data.clone();
    expected[[2, 3]].cell_type = CellType::None;
    expected[[3, 3]].cell_type = CellType::FireNormal;
    world.tmp = world.data.clone();
    world.update_fire();
    assert!(world.data == expected);
}
