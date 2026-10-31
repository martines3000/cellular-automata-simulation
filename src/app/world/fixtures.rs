use super::*;

pub(super) fn mixed_world(size: usize, seed: u64, burning: bool) -> World {
    assert!(size >= 3);
    let mut world = World::new(size);
    let mut rng = StdRng::seed_from_u64(seed);
    world.rng = StdRng::seed_from_u64(seed ^ 0x97a3_51dc_908b_6fe2);
    for y in 1..size - 1 {
        for x in 1..size - 1 {
            let material = match rng.gen_range(0..100) {
                0..=9 => CellType::Dirt,
                10..=24 => CellType::Sand,
                25..=34 => CellType::Wood,
                35..=59 => CellType::Water,
                60..=64 => CellType::Smoke,
                65..=69 => CellType::DarkSmoke,
                70..=72 if burning => CellType::FireNormal,
                73..=74 if burning => CellType::FireBurn,
                _ => CellType::None,
            };
            let cell = &mut world.data[[y, x]];
            cell.cell_type = material;
            cell.mass = match material {
                CellType::Water => {
                    let masses = [0.0, 0.005, MIN_MASS, 0.1, MIN_FLOW, 1.0, MAX_MASS, 25.0];
                    masses[rng.gen_range(0..masses.len())]
                }
                CellType::Smoke | CellType::DarkSmoke => {
                    let lifetimes = [0.0, 0.005, 0.01, 0.02, 1.0, 2.0];
                    lifetimes[rng.gen_range(0..lifetimes.len())]
                }
                _ => 0.0,
            };
        }
    }
    world.add_border();
    world
}

pub(super) fn basin_world(size: usize) -> World {
    let mut world = World::new(size);
    world.add_border();
    for y in size / 2..size - 1 {
        for x in 1..size - 1 {
            world.data[[y, x]].cell_type = CellType::Water;
            world.data[[y, x]].mass = MAX_MASS;
        }
    }
    world
}
