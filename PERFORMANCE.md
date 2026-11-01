# Local performance measurements

Use the pinned toolchain and lockfile. Correctness checks run with `bash check.sh`;
performance sampling is explicitly ignored by ordinary tests and runs locally only.
Windows support and CI are outside the backlog.

Before sampling, reserve the shared verification slot from plans/STATUS.md, inspect
worktrees/processes and use an otherwise idle machine. Run commands sequentially;
never overlap a GUI instance, compiler/test job or another benchmark.

Validate all benchmark inputs once, without reporting timings:

```sh
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_fixture_smoke_test -- --ignored --nocapture --test-threads=1
```

Collect measurements only in a quiet slot:

```sh
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_baseline -- --ignored --nocapture --test-threads=1
```

Divan 0.1.21 is a dev-only dependency with default features disabled; existing
application dependency versions are unchanged. The programmatic runner avoids
parsing libtest's flags. It requests one thread, 30 samples, one input per sample,
100 ms minimum and 1 s maximum per case including setup time for run limits.
Limits are checked between samples, not hard deadlines. Actual sample counts
may differ when the time cap applies. Release opt-level is the application's 2.

## What each case measures

All eight cases run at 100, 256, 512 and 1024 square cells. Seed 42 and a reset RNG
make mixed fixtures repeatable within the pinned rand version. Default app worlds
still use entropy. Every sample gets fresh input outside the measured closure;
tick samples cannot silently settle into an easier workload. Allocation of the
world and warm-up/layout are excluded. The shape vector retains capacity just as
it does in the app. No large traces/results are committed.

| Case | Timed operation |
| --- | --- |
| copy_grid | One data→scratch copy; each complete tick now performs six (including frozen materials). |
| tick_empty | One full tick with only Dirt border/empty interior. |
| tick_mixed | One full tick of the seeded all-material fixture. |
| shapes_mixed_full | Shape generation for the whole grid at one pixel/cell. |
| shapes_mixed_clipped | Shape generation in a fixed 600×600 viewport at five pixels/cell. |
| shapes_basin_full | Whole-grid shapes for a half-filled water basin. |
| tessellate_basin_full | egui CPU tessellation of prebuilt basin shapes; no input clone. |
| cpu_tick_and_ui | One explicit tick plus a settled, paused egui App frame; no GPU/presentation. |

The 2026-11-01 UI redesign changes `cpu_tick_and_ui`'s interface and its harness
window to 1280×820 points. Compare that case only with matching UI/viewport
conditions; historical UI cases are not isolated renderer comparisons. Core copy
and world-shape cases retain their workloads. The frozen-water follow-up
adds an Ice/Snow phase to every tick; historical tick/UI timings would therefore
compare different engine work. Existing mixed benchmark fixtures retain the old
material distribution, while frozen-material stress belongs to correctness tests;
add a separately named frozen benchmark when timing this feature is authorized.

Reported times exclude input setup, but consume output through black_box. Full-view
cases are scaling stress cases, not the default application zoom. Keep workload,
visible area, compiler/profile, lock hash, hardware and power/background conditions
constant when comparing versions. Repeat runs and report spread; do not gate tests
on machine-dependent timings. These results cannot establish interactive FPS,
GPU time or input latency.

## Verified structural changes

Two 1024² cell buffers use 16 MiB instead of 48 MiB, and the dense-basin fixture
emits 2,557 rectangles instead of 526,334. These are storage/geometry counts,
not timing improvements. [Exact evidence and limits](reviews/evidence/2026-10-31-scaling.md)
include the mixed-material regressions. Timed sampling is deferred at the user's
request; the [sampling plan](plans/planned/local-performance-measurements.md) is ready.

## Findings and options

[Tool research](reviews/research/2026-10-31-performance-tools.md) compares Divan,
Criterion, Embark Puffin and GPU options. The app already rasterizes with Glow on
the GPU. CPU cell-to-shape work, tessellation, mesh upload and simulation remain
separate costs. Consider GPU compute only after measuring a limiting CPU tick and
specifying conflict resolution/phase-order semantics. Native GPU/frame validation
is separate from this headless harness.
