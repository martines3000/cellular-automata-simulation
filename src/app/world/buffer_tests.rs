use super::*;

fn fixture() -> World {
    let mut world = World::new(7);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    for (y, x, material, mass) in [
        (3, 3, CellType::Water, 20.0),
        (2, 3, CellType::Wood, 0.0),
        (1, 3, CellType::Water, 5.0),
        (5, 1, CellType::FireBurn, 0.0),
        (5, 5, CellType::Smoke, 2.0),
    ] {
        world.data[[y, x]].cell_type = material;
        world.data[[y, x]].mass = mass;
    }
    world
}

#[test]
fn scratch_buffers_keep_their_original_allocations_across_ticks_and_edits() {
    let mut world = fixture();
    let pointers = [world.data.as_ptr(), world.tmp.as_ptr()];
    for _ in 0..20 {
        world.tick();
        assert!(pointers.contains(&world.data.as_ptr()));
        assert!(pointers.contains(&world.tmp.as_ptr()));
        assert_ne!(world.data.as_ptr(), world.tmp.as_ptr());
    }
    world.smooth();
    assert!(pointers.contains(&world.data.as_ptr()));
    assert!(pointers.contains(&world.tmp.as_ptr()));
}

#[test]
fn scratch_reuse_matches_the_original_clone_before_each_phase() {
    let mut world = fixture();
    let mut reference = fixture();
    for _ in 0..100 {
        world.tick();
        for update in [
            World::update_sand as fn(&mut World),
            World::update_water,
            World::update_fire,
            World::update_smoke,
            World::update_wood,
        ] {
            reference.tmp = reference.data.clone();
            update(&mut reference);
        }
        assert!(world.data == reference.data);
        assert!(world.tmp == reference.tmp);
    }
}
