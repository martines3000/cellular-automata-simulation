# Local performance tools and GPU options

Research date: 2026-10-31. Primary sources only; recommendations below are engineering judgments, not measured speedups. This note does not change the historical review or dependency snapshot. Verification and benchmark execution belong to the single owner in `plans/STATUS.md`. No CI or Windows work is proposed.

## Smallest suitable harness

Use **Divan `=0.1.21` as a dev dependency** for the new bounded local baseline. Its published docs specify Rust 1.80 or later, compatible with this project's pinned 1.95 toolchain. It supports parameterized workloads, counters and allocation instrumentation without requiring an application/framework migration. Criterion **0.8.2** is an alternative when statistical change estimates, saved baselines and charts become necessary; its default measurement is wall time. No broad dependency upgrade is a prerequisite. [Divan 0.1.21](https://docs.rs/divan/0.1.21/divan/), [Criterion 0.8.2](https://docs.rs/criterion/0.8.2/criterion/)

Keep benchmark functions in a private child module alongside `World` tests. Invoke Divan from one explicitly ignored Rust test; regular local checks must not run performance measurements. Source inspection supports this arrangement: the attribute preserves the private function and appends static registration, including Mach-O initialization on macOS. The parent subsequently confirmed compilation and all 32 cases through the non-timing smoke runner; measurement results remain separate. [Divan registration source](https://github.com/nvzqz/divan/blob/v0.1.21/macros/src/lib.rs)

An explicit runner avoids consuming libtest's command-line arguments:

```rust
#[test]
#[ignore = "local performance measurement; run alone in release mode"]
fn performance_baseline() {
    use std::time::Duration;
    divan::Divan::default()
        .threads([1])
        .sample_count(30)
        .sample_size(1)
        .min_time(Duration::from_millis(100))
        .max_time(Duration::from_secs(1))
        .skip_ext_time(false)
        .run_benches();
}
```

`Default` and `run_benches` do not parse CLI arguments; `from_args` and `config_with_args` do. The time ceiling is sampling control, not a preemptive timeout on a slow individual invocation. Leave Divan benchmark functions unignored; only the outer test needs `#[ignore]`. [Divan runner source](https://github.com/nvzqz/divan/blob/v0.1.21/src/divan.rs), [runner API](https://docs.rs/divan/0.1.21/divan/struct.Divan.html)

Use `with_inputs(|| fixture(size)).bench_local_refs(|world| ...)`: generating each fresh input is excluded from reported benchmark timing, and the local API accepts thread-bound inputs such as an egui harness. Each input is used once, preventing an evolving simulation from silently becoming an empty/settled benchmark. A sample size of one also limits retained world allocations per sample. Consume observable grid/shape results through `black_box`; it inhibits optimizer shortcuts on a best-effort basis. [Divan input/local API](https://docs.rs/divan/0.1.21/divan/struct.Bencher.html), [Rust black_box](https://doc.rust-lang.org/std/hint/fn.black_box.html)

## Workload and interpretation

- Compare bordered 100², 256² and 512² fixtures first; increase further only after observing runtime and memory. Keep larger sizes in tests/benchmarks before changing the interactive default.
- Separate complete simulation ticks, individual phases, shape generation and egui tessellation. Complete ticks must include the production scratch copies and sand → water → fire → smoke → wood schedule; phase-only timings must explicitly state whether scratch preparation is included.
- Build deterministic static layouts: empty, dense solids, supported water, mixed active water/wood/fire/smoke and sparse active regions. At baseline c737ec8 unconstrained sand used the thread RNG. This task switches to entropy-initialized StdRng and resets fixture seeds, making sand decisions reproducible within the pinned rand version.
- Reset outside the timed closure, warm the relevant paths, then time a fixed short tick sequence from identical inputs. Separately label any settled-state/long-running measurement. Test invariants and conservation budgets outside the timing interval.
- Record commit, dirty changes, lockfile hash, Rust version, release profile, hardware, power mode, workload, repetitions and timer output. Run alone with the shared verification slot; repeat comparable runs and report spread. Do not gate correctness on machine-dependent time thresholds.
- These are CPU wall-time measurements. They do not establish frame rate, GPU time, input latency or maximum interactive world size. Divan's summary statistics are useful descriptive results; Criterion's regression statistics can be added when noise-sensitive comparisons warrant them. [Criterion measurement/statistics scope](https://docs.rs/criterion/0.8.2/criterion/)

## Embark instrumentation

**Puffin 0.20.0** is a suitable later tool for locating expensive scopes during actual interactive frames: instrument tick phases, shape generation and surrounding UI work, enable collection explicitly and call `new_frame` once per frame. Scope instrumentation introduces overhead, so avoid one scope per cell. It supplements the repeatable harness rather than replacing it. [Puffin published API](https://docs.rs/puffin/0.20.0/puffin/), [Embark usage instructions](https://github.com/EmbarkStudios/puffin#how-to-use)

Do not install the current in-app `puffin_egui` viewer into this egui 0.17 app unchanged: current `puffin_egui` 0.30.0 follows workspace egui 0.37.0. A separate viewer/trace workflow avoids making a framework migration part of measurement; assess instrumentation overhead and trace size before adopting it. [Viewer manifest](https://github.com/EmbarkStudios/puffin/blob/main/puffin_egui/Cargo.toml), [workspace versions](https://github.com/EmbarkStudios/puffin/blob/main/Cargo.toml), [external viewer workflow](https://github.com/EmbarkStudios/puffin#remote-profiling)

## GPU decision

The application already uses GPU rasterization: eframe 0.17 defaults to `egui_glow`, whose OpenGL painter uploads mesh buffers and draws indexed triangles. egui first tessellates shapes on the CPU. Thus a slow grid display can come from CPU cell-to-shape work, tessellation, geometry upload, GPU work or presentation waits. A headless `gen_shapes` measurement covers only the first part. [eframe 0.17 features](https://github.com/emilk/egui/blob/0.17.0/eframe/Cargo.toml), [egui tessellation integration](https://github.com/emilk/egui/blob/0.17.0/egui/src/context.rs), [OpenGL painter](https://github.com/emilk/egui/blob/0.17.0/egui_glow/src/painter.rs)

First measure visible geometry count and tessellation. If those dominate, a CPU-generated color texture drawn as a quad is a bounded rendering experiment before GPU simulation. The existing backend supports texture uploads, including partial updates. A texture approach must preserve pan/zoom, nearest sampling and fractional water/wood geometry; a single texel per cell would change the current fractional surfaces. Upload cost and visual equivalence need actual native validation. This is a proposed experiment, not a claimed improvement. [Texture upload/filter implementation](https://github.com/emilk/egui/blob/0.17.0/egui_glow/src/painter.rs)

Consider GPU **compute** only if complete tick timings become the limiting factor at useful larger sizes. `wgpu` supports native Metal graphics and compute on macOS, but is not a drop-in accelerator for this old Glow integration. The current phases accumulate neighboring writes into `tmp`, depend on traversal/material eligibility and expose each phase to the next. A parallel implementation needs explicit conflict resolution, deterministic transfer budgets, ping-pong buffers and barriers between phases; naive one-thread-per-cell writes would race. Preserve the CPU model as an oracle before proposing a rule redesign. Moving every result back to CPU for shapes can negate compute gains; residency and renderer integration must be designed together. The suitability judgment is inferred from the current `World` implementation; backend capabilities come from [wgpu's official overview](https://wgpu.rs/).

No crate was added, no benchmarks were run and no GPU performance was measured while preparing this note. Published version/API checks are not compatibility or performance validation of this checkout.
