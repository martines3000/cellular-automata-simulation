use super::*;

fn water_mass(world: &World) -> f64 {
    world
        .data
        .iter()
        .map(|cell| f64::from(cell.water_quantity()))
        .sum()
}

fn solid_fixture() -> World {
    let mut world = World::new(5);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    world
}

#[test]
fn mixed_material_diagonal_sand_preserves_passive_water() {
    for x in [1, 3] {
        for residue in [0.0, MIN_MASS / 2.0, MIN_MASS] {
            let mut world = solid_fixture();
            world.data[[2, 2]].cell_type = CellType::Sand;
            world.data[[2, x]].cell_type = CellType::None;
            world.data[[3, x]].cell_type = CellType::None;
            world.data[[3, x]].mass = residue;
            world.tmp.assign(&world.data);
            world.update_sand();
            assert_eq!(water_mass(&world), f64::from(residue));
            assert_eq!(world.data[[3, x]].mass, 0.0);
            assert!(world.data[[3, x]].cell_type == CellType::Sand);
            assert_eq!(world.data[[2, 2]].mass, residue);
        }
    }
}

#[test]
fn mixed_material_falling_fire_preserves_passive_water() {
    for fire in [CellType::FireNormal, CellType::FireBurn] {
        for residue in [0.0, MIN_MASS / 2.0, MIN_MASS] {
            let mut world = solid_fixture();
            world.data[[2, 2]].cell_type = fire;
            world.data[[3, 2]].cell_type = CellType::None;
            world.data[[3, 2]].mass = residue;
            world.tmp.assign(&world.data);
            world.update_fire();
            assert_eq!(water_mass(&world), f64::from(residue));
            assert_eq!(world.data[[3, 2]].mass, 0.0);
            assert!(world.data[[3, 2]].cell_type == CellType::FireNormal);
            assert_eq!(world.data[[2, 2]].mass, residue);
        }
    }
}

#[test]
fn mixed_material_rising_smoke_preserves_passive_water() {
    for smoke in [CellType::Smoke, CellType::DarkSmoke] {
        for residue in [0.0, MIN_MASS / 2.0, MIN_MASS] {
            for lifetime in [0.005, 0.01, 1.0, 2.0] {
                let mut world = solid_fixture();
                world.data[[2, 2]].cell_type = smoke;
                world.data[[2, 2]].mass = lifetime;
                world.data[[1, 2]].cell_type = CellType::None;
                world.data[[1, 2]].mass = residue;
                world.tmp.assign(&world.data);
                world.update_smoke();
                assert_eq!(water_mass(&world), f64::from(residue));
                assert_eq!(world.data[[2, 2]].mass, residue);
                assert_eq!(world.data[[1, 2]].mass, (lifetime - 0.01).max(0.0));
                assert!(
                    world.data[[1, 2]].cell_type
                        == if lifetime > 0.01 {
                            smoke
                        } else {
                            CellType::None
                        }
                );
            }
        }
    }
}

fn count(world: &World, material: CellType) -> usize {
    world
        .data
        .iter()
        .filter(|cell| cell.cell_type == material)
        .count()
}

fn assert_state(world: &World, solids: &[((usize, usize), Cell)], context: &str) {
    for ((y, x), cell) in world.data.indexed_iter() {
        assert!(
            cell.mass.is_finite() && cell.mass >= 0.0,
            "{context}: invalid scalar at ({}, {}): {}",
            y,
            x,
            cell.mass
        );
        if cell.cell_type == CellType::None {
            assert!(
                cell.mass <= MIN_MASS,
                "{context}: inactive cell holds {} at ({}, {})",
                cell.mass,
                y,
                x
            );
        }
    }
    for (index, cell) in solids {
        assert!(world.data[*index] == *cell, "{context}: Dirt changed");
    }
}

pub(super) fn checked_tick(world: &mut World, context: &str) {
    let solids: Vec<_> = world
        .data
        .indexed_iter()
        .filter(|(_, cell)| cell.cell_type == CellType::Dirt)
        .map(|(index, cell)| (index, *cell))
        .collect();
    let sand = count(world, CellType::Sand);
    for (phase, update) in [
        ("sand", World::update_sand as fn(&mut World)),
        ("water", World::update_water),
        ("fire", World::update_fire),
        ("smoke", World::update_smoke),
        ("wood", World::update_wood),
        ("frozen", World::update_frozen),
    ] {
        let before = water_mass(world);
        let wood = count(world, CellType::Wood);
        let cleanup: f64 = if phase == "water" {
            world
                .data
                .iter()
                .filter(|cell| cell.cell_type == CellType::Water && cell.mass < MIN_MASS)
                .map(|cell| f64::from(cell.mass))
                .sum()
        } else {
            0.0
        };
        world.tmp.assign(&world.data);
        update(world);
        let context = format!("{context}, phase={phase}");
        assert_state(world, &solids, &context);
        assert_eq!(count(world, CellType::Sand), sand, "{context}: sand count");
        if phase != "fire" {
            assert_eq!(count(world, CellType::Wood), wood, "{context}: wood count");
        }
        // f32 additions/transfers round; intentional cleanup is budgeted separately.
        let rounding = 8.0 * f64::from(f32::EPSILON) * before.max(1.0);
        let after = water_mass(world);
        assert!(
            (after - (before - cleanup)).abs() <= rounding,
            "{context}: water {before} - cleanup {cleanup} != {after} (rounding {rounding})"
        );
    }
}

#[test]
fn mixed_material_seeded_ticks_preserve_invariants_and_match_production_schedule() {
    for burning in [false, true] {
        for size in [5, 7, 16] {
            for seed in 0..64 {
                let mut world = fixtures::mixed_world(size, seed, burning);
                let mut reference = fixtures::mixed_world(size, seed, burning);
                for tick in 0..24 {
                    let context =
                        format!("size={size}, seed={seed}, burning={burning}, tick={tick}");
                    checked_tick(&mut reference, &context);
                    world.tick();
                    assert!(world.data == reference.data, "{context}: production data");
                    assert!(world.tmp == reference.tmp, "{context}: production scratch");
                }
            }
        }
    }
}

#[test]
fn mixed_material_large_worlds_preserve_state_and_borders() {
    for size in [100, 256, 512, 1024] {
        let mut world = fixtures::mixed_world(size, 42, true);
        for tick in 0..2 {
            checked_tick(&mut world, &format!("size={size}, tick={tick}"));
        }
    }
}
