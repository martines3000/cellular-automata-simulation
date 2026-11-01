use super::*;

fn smoke_world() -> World {
    let mut world = World::new(7);
    for cell in &mut world.data {
        cell.cell_type = CellType::Dirt;
    }
    world.data[[3, 3]].cell_type = CellType::Smoke;
    world.data[[3, 3]].mass = 7.0;
    world
}

#[test]
fn timing_update_catches_up_multiple_ticks_after_elapsed_time() {
    let mut world = smoke_world();
    // At least 100 ms has elapsed, independently of scheduling delays; no sleep.
    world.last_frame_time = Instant::now() - Duration::from_millis(100);
    world.update();
    assert!(
        world.data[[3, 3]].mass <= 6.98,
        "100 ms at 60 steps/s must advance more than one tick; smoke mass {}",
        world.data[[3, 3]].mass
    );
}

#[test]
fn timing_same_elapsed_time_matches_at_different_repaint_cadences() {
    for rate in [1, 30, 60, 120, 360] {
        let mut expected = smoke_world();
        for _ in 0..rate {
            expected.tick();
        }
        for frames in [30, 60, 120, 144] {
            let mut world = smoke_world();
            world.steps_per_second = rate;
            let mut previous = 0;
            let mut ticks = 0;
            for frame in 1..=frames {
                let now = NANOS_PER_SECOND * frame / frames;
                ticks += world.advance(Duration::from_nanos((now - previous) as u64));
                previous = now;
            }
            assert_eq!(ticks, rate as usize, "rate={rate}, frames={frames}");
            assert_eq!(world.step_credit, 0);
            assert!(world.data == expected.data);
        }
    }
}

#[test]
fn timing_nanosecond_boundaries_for_every_supported_rate() {
    for rate in 1..=360 {
        let mut world = smoke_world();
        world.steps_per_second = rate;
        let below_tick = (NANOS_PER_SECOND - 1) / rate as u128;
        assert_eq!(world.advance(Duration::from_nanos(below_tick as u64)), 0);
        assert_eq!(world.advance(Duration::from_nanos(1)), 1);
        assert!(world.step_credit < NANOS_PER_SECOND);
        assert_eq!(world.data[[3, 3]].mass, 7.0 - 0.01);
    }
}

#[test]
fn timing_fractional_ticks_accumulate_instead_of_resetting() {
    let mut world = smoke_world();
    let ticks: Vec<usize> = (0..5)
        .map(|_| world.advance(Duration::from_millis(10)))
        .collect();
    assert_eq!(ticks, [0, 1, 0, 1, 1]);
    assert_eq!(world.step_credit, 0); // 50 ms × 60 steps/s = exactly three ticks.
    let mut expected = smoke_world();
    for _ in 0..3 {
        expected.tick();
    }
    assert!(world.data == expected.data);
}

#[test]
fn timing_stall_caps_catch_up_drops_whole_debt_and_keeps_fraction() {
    let mut world = smoke_world();
    assert_eq!(
        world.advance(Duration::from_secs(100) + Duration::from_millis(1)),
        MAX_STEPS_PER_UPDATE
    );
    assert_eq!(world.step_credit, 60_000_000);
    assert_eq!(world.advance(Duration::ZERO), 0);
    assert_eq!(world.advance(Duration::from_nanos(15_666_666)), 0);
    assert_eq!(world.advance(Duration::from_nanos(1)), 1);
    let mut expected = smoke_world();
    for _ in 0..MAX_STEPS_PER_UPDATE + 1 {
        expected.tick();
    }
    assert!(world.data == expected.data);
}

#[test]
fn timing_maximum_duration_does_not_overflow_or_leave_whole_debt() {
    let mut world = smoke_world();
    world.steps_per_second = 360;
    for _ in 0..3 {
        assert_eq!(world.advance(Duration::MAX), MAX_STEPS_PER_UPDATE);
        assert!(world.step_credit < NANOS_PER_SECOND);
        assert_eq!(world.advance(Duration::ZERO), 0);
    }
    assert!(world
        .data
        .iter()
        .all(|cell| cell.mass.is_finite() && cell.mass >= 0.0));
}

#[test]
fn timing_invalid_programmatic_rates_are_clamped_to_slider_range() {
    for rate in [i32::MIN, -1, 0, 1, 60, 360, 361, i32::MAX] {
        let mut world = smoke_world();
        world.steps_per_second = rate;
        assert_eq!(world.advance(Duration::ZERO), 0);
        assert_eq!(world.steps_per_second, rate.clamp(1, 360));
        let one_tick = (NANOS_PER_SECOND - 1) / world.steps_per_second as u128 + 1;
        assert_eq!(world.advance(Duration::from_nanos(one_tick as u64)), 1);
    }
}

#[test]
fn timing_pause_resume_reset_discards_fractional_credit() {
    let mut world = smoke_world();
    assert_eq!(world.advance(Duration::from_millis(10)), 0);
    assert!(world.step_credit > 0);
    world.reset_timing();
    assert_eq!(world.step_credit, 0);
    assert_eq!(world.advance(Duration::ZERO), 0);
    assert_eq!(world.advance(Duration::from_millis(10)), 0);
    assert_eq!(world.data[[3, 3]].mass, 7.0);
}

#[test]
fn timing_rate_change_starts_a_fresh_interval() {
    for (old_rate, new_rate) in [(60, 120), (120, 60), (1, 360), (360, 1)] {
        let mut world = smoke_world();
        world.steps_per_second = old_rate;
        assert_eq!(world.advance(Duration::from_nanos(1)), 0);
        world.steps_per_second = new_rate;
        world.reset_timing();
        let below_tick = (NANOS_PER_SECOND - 1) / new_rate as u128;
        assert_eq!(world.advance(Duration::from_nanos(below_tick as u64)), 0);
        assert_eq!(world.advance(Duration::from_nanos(1)), 1);
    }
}

#[test]
fn timing_due_tick_preserves_production_material_phase_order() {
    fn fixture() -> World {
        let mut world = World::new(7);
        world.add_border();
        for (y, x, material, mass) in [
            (2, 1, CellType::Sand, 0.0),
            (2, 3, CellType::Water, 1.0),
            (3, 3, CellType::Water, 0.005),
            (4, 1, CellType::FireBurn, 0.0),
            (5, 1, CellType::Dirt, 0.0),
            (4, 2, CellType::Wood, 0.0),
            (4, 4, CellType::Smoke, 1.0),
            (4, 5, CellType::Wood, 0.0),
        ] {
            world.data[[y, x]].cell_type = material;
            world.data[[y, x]].mass = mass;
        }
        world
    }
    let mut expected = fixture();
    for update in [
        World::update_sand as fn(&mut World),
        World::update_water,
        World::update_fire,
        World::update_smoke,
        World::update_wood,
        World::update_frozen,
        World::update_steam,
    ] {
        expected.tmp = expected.data.clone();
        update(&mut expected);
    }
    let mut world = fixture();
    world.steps_per_second = 1;
    assert_eq!(world.advance(Duration::from_secs(1)), 1);
    assert!(world.data == expected.data);
}
