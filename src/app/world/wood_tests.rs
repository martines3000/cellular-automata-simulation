use super::*;

fn fluid_mass(grid: &Array2<Cell>) -> f64 {
    grid.iter()
        .filter(|cell| matches!(cell.cell_type, CellType::Water | CellType::None))
        .map(|cell| f64::from(cell.mass))
        .sum()
}

fn wood_count(grid: &Array2<Cell>) -> usize {
    grid.iter()
        .filter(|cell| cell.cell_type == CellType::Wood)
        .count()
}

fn sealed_world() -> World {
    let mut world = World::new(7);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    world
}

fn assert_water_phase(world: &mut World, expected_mass: f64, expected_wood: usize) {
    let solids: Vec<Cell> = world
        .data
        .iter()
        .filter(|cell| cell.cell_type == CellType::Dirt)
        .copied()
        .collect();
    world.tmp = world.data.clone();
    world.update_water();
    assert_eq!(wood_count(&world.data), expected_wood);
    let rounding = 4.0 * f64::from(f32::EPSILON) * expected_mass.max(1.0);
    assert!(
        (fluid_mass(&world.data) - expected_mass).abs() <= rounding,
        "mass {} != {expected_mass}",
        fluid_mass(&world.data)
    );
    assert!(world
        .data
        .iter()
        .all(|cell| cell.mass.is_finite() && cell.mass >= 0.0));
    for cell in solids {
        assert!(world.data[[cell.y, cell.x]] == cell);
    }
}

#[test]
fn wood_displacement_lift_preserves_destination_water_and_stacks() {
    for height in 1..=4 {
        for (destination_type, mass) in [
            (CellType::None, 0.0),
            (CellType::None, MIN_MASS / 2.0),
            (CellType::Water, 0.0),
            (CellType::Water, MIN_MASS / 2.0),
            (CellType::Water, MIN_MASS),
            (CellType::Water, 5.0),
            (CellType::Water, 20.0),
        ] {
            let mut world = sealed_world();
            let top = 5 - height;
            for y in top + 1..=5 {
                world.data[[y, 3]].cell_type = CellType::Wood;
            }
            world.data[[top, 3]].cell_type = destination_type;
            world.data[[top, 3]].mass = mass;
            world.tmp = world.data.clone();
            assert_eq!(world.wood_stuck(5, 3), (false, top));
            world.move_wood([5, 3], [top, 3]);
            assert_eq!(fluid_mass(&world.tmp), f64::from(mass));
            assert_eq!(wood_count(&world.tmp), height);
            assert!(world.tmp[[top, 3]].cell_type == CellType::Wood);
            assert_eq!(world.tmp[[top, 3]].mass, 0.0);
            assert_eq!(world.tmp[[5, 3]].mass, mass);
            assert!(
                world.tmp[[5, 3]].cell_type
                    == if mass >= MIN_MASS {
                        CellType::Water
                    } else {
                        CellType::None
                    }
            );
        }
    }
}

#[test]
fn wood_displacement_pressure_preserves_saved_twenty_five_unit_case() {
    let mut world = sealed_world();
    world.data[[3, 3]].cell_type = CellType::Water;
    world.data[[3, 3]].mass = 20.0;
    world.data[[2, 3]].cell_type = CellType::Wood;
    world.data[[1, 3]].cell_type = CellType::Water;
    world.data[[1, 3]].mass = 5.0;
    for _ in 0..16 {
        assert_water_phase(&mut world, 25.0, 1);
    }
}

#[test]
fn wood_displacement_side_lift_preserves_destination_water() {
    let mut world = sealed_world();
    world.data[[3, 3]].cell_type = CellType::Water;
    world.data[[3, 3]].mass = 20.0;
    world.data[[3, 4]].cell_type = CellType::Wood;
    world.data[[2, 4]].cell_type = CellType::Water;
    world.data[[2, 4]].mass = 5.0;
    assert_water_phase(&mut world, 25.0, 1);
    assert!(world.data[[2, 4]].cell_type == CellType::Wood);
    assert!(world.data[[3, 4]].cell_type == CellType::Water);
}

#[test]
fn wood_displacement_blocked_stack_preserves_state() {
    let mut world = sealed_world();
    world.data[[3, 3]].cell_type = CellType::Water;
    world.data[[3, 3]].mass = 20.0;
    world.data[[2, 3]].cell_type = CellType::Wood;
    let before = world.data.clone();
    assert_water_phase(&mut world, 20.0, 1);
    assert!(world.data == before);
}

#[test]
fn lateral_flow_symmetric_source_splits_equally_at_processing_threshold() {
    for mass in [
        f32::from_bits(MIN_FLOW.to_bits() - 1),
        MIN_FLOW,
        f32::from_bits(MIN_FLOW.to_bits() + 1),
        1.0,
        10.0,
        20.0,
    ] {
        let mut world = sealed_world();
        world.data[[3, 3]].cell_type = CellType::Water;
        world.data[[3, 3]].mass = mass;
        world.data[[3, 2]].cell_type = CellType::None;
        world.data[[3, 4]].cell_type = CellType::None;
        assert_water_phase(&mut world, f64::from(mass), 0);
        assert_eq!(world.data[[3, 2]].mass, world.data[[3, 4]].mass);
        if mass == 10.0 {
            assert_eq!(world.data[[3, 2]].mass, 2.5);
        }
    }
}

fn lateral_fixture(left: usize, right: usize, mass: f32) -> World {
    let mut world = sealed_world();
    world.data[[3, 3]].cell_type = CellType::Water;
    world.data[[3, 3]].mass = mass;
    for (x, beyond, state) in [(2, 1, left), (4, 5, right)] {
        world.data[[3, x]].cell_type = match state {
            0 => CellType::None,
            1 => CellType::Water,
            2 | 3 => CellType::Wood,
            _ => CellType::Dirt,
        };
        if state == 2 {
            world.data[[3, beyond]].cell_type = CellType::None;
        }
    }
    world
}

#[test]
fn lateral_flow_material_matrix_is_mirrored_and_conservative() {
    for left in 0..5 {
        for right in 0..5 {
            for mass in [MIN_FLOW, 1.0, 10.0, 20.0] {
                let mut world = lateral_fixture(left, right, mass);
                let mut mirrored = lateral_fixture(right, left, mass);
                let woods = wood_count(&world.data);
                assert_water_phase(&mut world, f64::from(mass), woods);
                assert_water_phase(&mut mirrored, f64::from(mass), woods);
                for y in 0..7 {
                    for x in 0..7 {
                        assert!(
                            world.data[[y, x]].cell_type == mirrored.data[[y, 6 - x]].cell_type,
                            "left={left}, right={right}, mass={mass}, y={y}, x={x}"
                        );
                        assert_eq!(world.data[[y, x]].mass, mirrored.data[[y, 6 - x]].mass);
                    }
                }
            }
        }
    }
}

#[test]
fn lateral_flow_zero_does_not_move_wood_or_convert_its_scalar_to_water() {
    for (x, beyond) in [(2, 1), (4, 5)] {
        let mut world = sealed_world();
        world.data[[3, 3]].cell_type = CellType::Water;
        world.data[[3, 3]].mass = 1.0;
        world.data[[3, x]].cell_type = CellType::Wood;
        world.data[[3, x]].mass = 2.0;
        world.data[[3, beyond]].cell_type = CellType::None;
        let before = world.data.clone();
        world.tmp = world.data.clone();
        world.update_water();
        assert!(world.data == before);
    }
}

#[test]
fn lateral_flow_wood_into_passive_water_preserves_quantity() {
    for (x, beyond) in [(2, 1), (4, 5)] {
        let mut world = sealed_world();
        world.data[[3, 3]].cell_type = CellType::Water;
        world.data[[3, 3]].mass = 1.0;
        world.data[[3, x]].cell_type = CellType::Wood;
        world.data[[3, beyond]].cell_type = CellType::None;
        world.data[[3, beyond]].mass = MIN_MASS / 2.0;
        assert_water_phase(&mut world, 1.0 + f64::from(MIN_MASS / 2.0), 1);
        assert!(world.data[[3, beyond]].cell_type == CellType::Wood);
        assert_eq!(world.data[[3, beyond]].mass, 0.0);
    }
}

#[test]
fn lateral_flow_near_borders_lifts_wood_without_losing_water() {
    for x in 1..6 {
        let mut world = sealed_world();
        world.data[[3, x]].cell_type = CellType::Water;
        world.data[[3, x]].mass = 20.0;
        let mut woods = 0;
        for nx in [x - 1, x + 1] {
            if (1..6).contains(&nx) {
                world.data[[3, nx]].cell_type = CellType::Wood;
                world.data[[2, nx]].cell_type = CellType::Water;
                world.data[[2, nx]].mass = 1.0;
                woods += 1;
            }
        }
        assert_water_phase(&mut world, 20.0 + woods as f64, woods);
        for nx in [x - 1, x + 1] {
            if (1..6).contains(&nx) {
                assert!(world.data[[2, nx]].cell_type == CellType::Wood);
                assert_eq!(world.data[[2, nx]].mass, 0.0);
            }
        }
    }
}

#[test]
fn lateral_flow_blocked_wood_does_not_skip_upward_pressure() {
    for x in [2, 4] {
        let mut world = sealed_world();
        world.data[[3, 3]].cell_type = CellType::Water;
        world.data[[3, 3]].mass = 20.0;
        world.data[[3, x]].cell_type = CellType::Wood;
        world.data[[2, 3]].cell_type = CellType::None;
        assert_water_phase(&mut world, 20.0, 1);
        assert!(world.data[[3, x]].cell_type == CellType::Wood);
        assert_eq!(world.data[[2, 3]].mass, 3.0);
        assert!(world.data[[2, 3]].cell_type == CellType::Water);
    }
}

#[test]
fn wood_displacement_falling_preserves_passive_destination_water() {
    for mass in [0.0, MIN_MASS / 2.0, MIN_MASS] {
        let mut world = sealed_world();
        world.data[[3, 3]].cell_type = CellType::Wood;
        world.data[[4, 3]].cell_type = CellType::None;
        world.data[[4, 3]].mass = mass;
        world.tmp = world.data.clone();
        world.update_wood();
        assert_eq!(fluid_mass(&world.data), f64::from(mass));
        assert_eq!(wood_count(&world.data), 1);
        assert!(world.data[[4, 3]].cell_type == CellType::Wood);
        assert_eq!(world.data[[4, 3]].mass, 0.0);
    }
}

#[test]
fn wood_displacement_full_tick_schedule_preserves_closed_fixture() {
    let mut world = sealed_world();
    world.data[[3, 3]].cell_type = CellType::Water;
    world.data[[3, 3]].mass = 20.0;
    world.data[[2, 3]].cell_type = CellType::Wood;
    world.data[[1, 3]].cell_type = CellType::Water;
    world.data[[1, 3]].mass = 5.0;
    let solids: Vec<Cell> = world
        .data
        .iter()
        .filter(|cell| cell.cell_type == CellType::Dirt)
        .copied()
        .collect();
    for _ in 0..32 {
        for update in [
            World::update_sand as fn(&mut World),
            World::update_water,
            World::update_fire,
            World::update_smoke,
            World::update_wood,
        ] {
            world.tmp = world.data.clone();
            update(&mut world);
        }
        assert_eq!(wood_count(&world.data), 1);
        assert!((fluid_mass(&world.data) - 25.0).abs() <= 4.0 * f64::from(f32::EPSILON) * 25.0);
        assert!(world
            .data
            .iter()
            .all(|cell| cell.mass.is_finite() && cell.mass >= 0.0));
        for cell in &solids {
            assert!(world.data[[cell.y, cell.x]] == *cell);
        }
    }
}
