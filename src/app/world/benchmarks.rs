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
            harness.canvas(); // Settle layout outside timing; UI remains paused.
            harness
        })
        .bench_local_refs(|harness| {
            harness.app.world.tick();
            black_box(harness.input(vec![]))
        });
}
