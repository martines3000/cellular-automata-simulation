use super::*;
use crate::app::tests::Harness;
use divan::Bencher;
use std::hint::black_box;

#[test]
#[ignore = "local performance measurement; run alone in release mode"]
fn performance_baseline() {
    if cfg!(debug_assertions) {
        panic!("use cargo test --release");
    }
    eprintln!(
        "Cell: {} bytes; release opt-level=2; fresh input per sample",
        mem::size_of::<Cell>()
    );
    divan::Divan::default()
        .threads([1])
        .sample_count(30)
        .sample_size(1)
        .min_time(Duration::from_millis(100))
        .max_time(Duration::from_secs(1))
        .skip_ext_time(false)
        .run_benches();
}

#[test]
#[ignore = "validate benchmark fixtures separately from normal checks"]
fn performance_fixture_smoke_test() {
    eprintln!("Cell storage: {} bytes", mem::size_of::<Cell>());
    divan::Divan::default().threads([1]).test_benches();
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn tick_empty(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| {
            let mut world = World::new(size);
            world.add_border();
            world
        })
        .bench_local_refs(|world| {
            world.tick();
            black_box(&world.data);
        });
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn tick_mixed(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| fixtures::mixed_world(size, 42, true))
        .bench_local_refs(|world| {
            world.tick();
            black_box(&world.data);
        });
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn copy_grid(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| fixtures::mixed_world(size, 42, true))
        .bench_local_refs(|world| {
            world.tmp.assign(black_box(&world.data));
            black_box(&world.tmp);
        });
}

fn drawing_input(size: usize, basin: bool, full_view: bool) -> (World, Vec<Shape>, Rect) {
    let mut world = if basin {
        fixtures::basin_world(size)
    } else {
        fixtures::mixed_world(size, 42, true)
    };
    world.use_shift = true;
    let viewport = if full_view {
        world.block_size = 1.0;
        Rect::from_min_size(pos2(0.0, 0.0), vec2(size as f32, size as f32))
    } else {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 600.0))
    };
    let mut shapes = Vec::new();
    world.gen_shapes(&mut shapes, viewport);
    shapes.clear(); // Match the application's retained vector capacity.
    (world, shapes, viewport)
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn shapes_mixed_full(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| drawing_input(size, false, true))
        .bench_local_refs(|(world, shapes, viewport)| {
            world.gen_shapes(shapes, *viewport);
            black_box(&shapes);
        });
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn shapes_mixed_clipped(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| drawing_input(size, false, false))
        .bench_local_refs(|(world, shapes, viewport)| {
            world.gen_shapes(shapes, *viewport);
            black_box(&shapes);
        });
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn shapes_basin_full(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| drawing_input(size, true, true))
        .bench_local_refs(|(world, shapes, viewport)| {
            world.gen_shapes(shapes, *viewport);
            black_box(&shapes);
        });
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn tessellate_basin_full(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| {
            let (world, mut shapes, viewport) = drawing_input(size, true, true);
            world.gen_shapes(&mut shapes, viewport);
            let ctx = egui::Context::default();
            ctx.begin_frame(egui::RawInput::default());
            let _ = ctx.end_frame();
            let clipped: Vec<_> = shapes
                .into_iter()
                .map(|shape| egui::epaint::ClippedShape(viewport, shape))
                .collect();
            (ctx, clipped)
        })
        .bench_local_refs(|(ctx, shapes)| black_box(ctx.tessellate(mem::take(shapes))));
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn cpu_tick_and_ui(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| {
            let mut harness = Harness::new();
            harness.app.world = fixtures::mixed_world(size, 42, true);
            harness.app.pending_world_size = size;
            harness.canvas(); // Settle layout outside timing; UI remains paused.
            harness
        })
        .bench_local_refs(|harness| {
            harness.app.world.tick();
            black_box(harness.input(vec![]))
        });
}

// Keep the original mixed workload stable; explicitly exercise newer materials.
fn materials_world(size: usize) -> World {
    let mut world = fixtures::mixed_world(size, 42, true);
    for y in 1..size - 1 {
        for x in 1..size - 1 {
            if (x + y * 3) % 11 < 3 {
                world.data[[y, x]] = Cell {
                    cell_type: [CellType::Ice, CellType::Snow, CellType::Steam][(x + y * 3) % 11],
                    mass: if x % 2 == 0 { 1.0 } else { MAX_MASS },
                };
            }
        }
    }
    world
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn tick_materials(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| materials_world(size))
        .bench_local_refs(|world| {
            world.tick();
            black_box(&world.data);
        });
}

macro_rules! phase_bench {
    ($name:ident, $phase:ident) => {
        #[divan::bench(args = [100, 256, 512, 1024])]
        fn $name(bencher: Bencher, size: usize) {
            bencher
                .with_inputs(|| materials_world(size))
                .bench_local_refs(|world| {
                    world.tmp.assign(&world.data);
                    world.$phase();
                    black_box(&world.data);
                });
        }
    };
}
phase_bench!(phase_sand, update_sand);
phase_bench!(phase_water, update_water);
phase_bench!(phase_fire, update_fire);
phase_bench!(phase_smoke, update_smoke);
phase_bench!(phase_wood, update_wood);
phase_bench!(phase_frozen, update_frozen);
phase_bench!(phase_steam, update_steam);

#[divan::bench(args = [100, 256, 512, 1024])]
fn overview_colors_full(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| drawing_input(size, false, true))
        .bench_local_refs(|(world, _, view)| {
            black_box(crate::app::render::overview(world, *view));
        });
}

fn overview_ui(bencher: Bencher, size: usize, texture: bool) {
    bencher
        .with_inputs(|| {
            let mut h = Harness::new();
            h.app.world = fixtures::mixed_world(size, 42, true);
            h.app.pending_world_size = size;
            h.app.fast_overview = texture;
            h.app.fit_view = true;
            h.canvas();
            h
        })
        .bench_local_refs(|h| black_box(h.input(vec![])));
}

#[divan::bench(args = [100, 256, 512, 1024])]
fn overview_ui_texture(bencher: Bencher, size: usize) {
    overview_ui(bencher, size, true);
}
#[divan::bench(args = [100, 256, 512, 1024])]
fn overview_ui_geometry(bencher: Bencher, size: usize) {
    overview_ui(bencher, size, false);
}
