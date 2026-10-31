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
            world.lift_wood(5, 3, top);
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
